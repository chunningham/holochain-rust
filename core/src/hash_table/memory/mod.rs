use std::collections::HashMap;

use error::HolochainError;

use hash_table::{
    pair::Pair,
    pair_meta::PairMeta,
    HashTable,
};

#[derive(Serialize, Debug, Clone, PartialEq, Default)]
pub struct MemTable {
    pairs: HashMap<String, Pair>,
    meta: HashMap<String, PairMeta>,
}

impl MemTable {
    pub fn new() -> MemTable {
        MemTable {
            pairs: HashMap::new(),
            meta: HashMap::new(),
        }
    }
}

impl HashTable for MemTable {

    fn commit_pair(&mut self, pair: &Pair) -> Result<(), HolochainError> {
        self.pairs.insert(pair.key(), pair.clone());
        Ok(())
    }

    fn pair(&self, key: &str) -> Result<Option<Pair>, HolochainError> {
        Ok(self.pairs.get(key).cloned())
    }

    fn assert_pair_meta(&mut self, meta: PairMeta) -> Result<(), HolochainError> {
        self.meta.insert(meta.key(), meta);
        Ok(())
    }

    fn pair_meta(&mut self, key: &str) -> Result<Option<PairMeta>, HolochainError> {
        Ok(self.meta.get(key).cloned())
    }

    fn all_metas_for_pair(&mut self, pair: &Pair) -> Result<Vec<PairMeta>, HolochainError> {
        let mut metas = self
            .meta
            .values()
            .filter(|&m| m.pair() == pair.key())
            .cloned()
            .collect::<Vec<PairMeta>>();
        // @TODO should this be sorted at all at this point?
        // @see https://github.com/holochain/holochain-rust/issues/144
        metas.sort();
        Ok(metas)
    }
}

#[cfg(test)]
pub mod tests {

    use agent::keys::tests::test_keys;
    use hash_table::{
        memory::MemTable,
        pair::tests::{test_pair, test_pair_a, test_pair_b},
        pair_meta::{
            tests::{test_pair_meta, test_pair_meta_a, test_pair_meta_b},
            PairMeta,
        },
        status::{CRUDStatus, LINK_NAME, STATUS_NAME},
        HashTable,
    };
    use hash_table::test_util::test_round_trip;

    pub fn test_table() -> MemTable {
        MemTable::new()
    }

    #[test]
    /// smoke test
    fn new() {
        test_table();
    }

    #[test]
    /// tests for ht.setup()
    fn setup() {
        let mut ht = test_table();
        assert_eq!(Ok(()), ht.setup());
    }

    #[test]
    /// tests for ht.teardown()
    fn teardown() {
        let mut ht = test_table();
        assert_eq!(Ok(()), ht.teardown());
    }

    #[test]
    /// Pairs can round trip through table.commit() and table.get()
    fn pair_round_trip() {
        test_round_trip(&mut test_table());
    }

    #[test]
    /// Pairs can be modified through table.modify()
    fn modify() {
        let mut ht = test_table();
        let p1 = test_pair_a();
        let p2 = test_pair_b();

        ht.commit_pair(&p1).expect("should be able to commit valid pair");
        ht.modify_pair(&test_keys(), &p1, &p2)
            .expect("should be able to edit with valid pair");

        assert_eq!(
            vec![
                PairMeta::new(&test_keys(), &p1, LINK_NAME, &p2.key()),
                PairMeta::new(
                    &test_keys(),
                    &p1,
                    STATUS_NAME,
                    &CRUDStatus::MODIFIED.bits().to_string(),
                ),
            ],
            ht.all_metas_for_pair(&p1)
                .expect("getting the metadata on a pair shouldn't fail")
        );

        let empty_vec: Vec<PairMeta> = Vec::new();
        assert_eq!(
            empty_vec,
            ht.all_metas_for_pair(&p2)
                .expect("getting the metadata on a pair shouldn't fail")
        );
    }

    #[test]
    /// Pairs can be retracted through table.retract()
    fn retract_pair() {
        let mut table = test_table();
        let pair = test_pair();
        let empty_vec: Vec<PairMeta> = Vec::new();

        table.commit_pair(&pair).expect("should be able to commit valid pair");
        assert_eq!(
            empty_vec,
            table.all_metas_for_pair(&pair)
                .expect("getting the metadata on a pair shouldn't fail")
        );

        table.retract_pair(&test_keys(), &pair)
            .expect("should be able to retract");
        assert_eq!(
            vec![PairMeta::new(
                &test_keys(),
                &pair,
                STATUS_NAME,
                &CRUDStatus::DELETED.bits().to_string(),
            )],
            table.all_metas_for_pair(&pair)
                .expect("getting the metadata on a pair shouldn't fail"),
        );
    }

    #[test]
    /// PairMeta can round trip through table.assert_meta() and table.get_meta()
    fn meta_round_trip() {
        let mut table = test_table();
        let m = test_pair_meta();

        assert_eq!(
            None,
            table.pair_meta(&m.key())
                .expect("getting the metadata on a pair shouldn't fail")
        );

        table.assert_pair_meta(m.clone())
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            Some(&m),
            table.pair_meta(&m.key())
                .expect("getting the metadata on a pair shouldn't fail")
                .as_ref()
        );
    }

    #[test]
    /// all PairMeta for a Pair can be retrieved with all_metas_for_pair
    fn all_metas_for_pair() {
        let mut ht = test_table();
        let p = test_pair();
        let m1 = test_pair_meta_a();
        let m2 = test_pair_meta_b();
        let empty_vec: Vec<PairMeta> = Vec::new();

        assert_eq!(
            empty_vec,
            ht.all_metas_for_pair(&p)
                .expect("getting the metadata on a pair shouldn't fail")
        );

        ht.assert_pair_meta(m1.clone())
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            vec![m1.clone()],
            ht.all_metas_for_pair(&p)
                .expect("getting the metadata on a pair shouldn't fail")
        );

        ht.assert_pair_meta(m2.clone())
            .expect("asserting metadata shouldn't fail");
        assert_eq!(
            vec![m2, m1],
            ht.all_metas_for_pair(&p)
                .expect("getting the metadata on a pair shouldn't fail")
        );
    }
}
