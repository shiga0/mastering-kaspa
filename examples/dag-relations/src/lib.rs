use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub struct Dag {
    // child -> parents
    parents: BTreeMap<String, BTreeSet<String>>,
}

impl Dag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, id: &str, parents: &[&str]) -> Result<(), String> {
        if self.parents.contains_key(id) {
            return Err(format!("duplicate block: {id}"));
        }
        if parents.contains(&id) {
            return Err("self-parent is not allowed".into());
        }
        if parents.iter().any(|p| !self.parents.contains_key(*p)) {
            return Err("unknown parent".into());
        }
        self.parents.insert(
            id.to_owned(),
            parents.iter().map(|p| (*p).to_owned()).collect(),
        );
        Ok(())
    }

    pub fn past(&self, id: &str) -> Result<BTreeSet<String>, String> {
        if !self.parents.contains_key(id) {
            return Err(format!("unknown block: {id}"));
        }
        let mut visited = BTreeSet::new();
        let mut stack: Vec<String> = self.parents[id].iter().cloned().collect();
        while let Some(next) = stack.pop() {
            if visited.insert(next.clone()) {
                stack.extend(self.parents[&next].iter().cloned());
            }
        }
        Ok(visited)
    }

    pub fn anticone(&self, id: &str) -> Result<BTreeSet<String>, String> {
        let past = self.past(id)?;
        let mut result = BTreeSet::new();
        for other in self.parents.keys() {
            if other == id || past.contains(other) {
                continue;
            }
            // id が other の祖先なら、other は id の future に属する。
            if self.past(other)?.contains(id) {
                continue;
            }
            result.insert(other.clone());
        }
        Ok(result)
    }

    pub fn is_k_cluster(&self, selected: &BTreeSet<String>, k: usize) -> Result<bool, String> {
        for id in selected {
            let anticone = self.anticone(id)?;
            let within = anticone.intersection(selected).count();
            if within > k {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Dag {
        let mut dag = Dag::new();
        dag.insert("A", &[]).unwrap();
        dag.insert("B", &["A"]).unwrap();
        dag.insert("C", &["B"]).unwrap();
        dag.insert("D", &["B"]).unwrap();
        dag.insert("E", &["C", "D"]).unwrap();
        dag
    }

    #[test]
    fn past_contains_transitive_ancestors() {
        let dag = sample();
        assert_eq!(
            dag.past("E").unwrap(),
            BTreeSet::from(["A", "B", "C", "D"].map(str::to_owned))
        );
    }

    #[test]
    fn parallel_blocks_are_in_each_others_anticone() {
        let dag = sample();
        assert_eq!(dag.anticone("C").unwrap(), BTreeSet::from(["D".to_owned()]));
        assert_eq!(dag.anticone("D").unwrap(), BTreeSet::from(["C".to_owned()]));
    }

    #[test]
    fn k_cluster_constraint() {
        let dag = sample();
        let selected = BTreeSet::from(["C".to_owned(), "D".to_owned()]);
        assert!(!dag.is_k_cluster(&selected, 0).unwrap());
        assert!(dag.is_k_cluster(&selected, 1).unwrap());
    }

    #[test]
    fn rejects_missing_parent() {
        let mut dag = Dag::new();
        assert!(dag.insert("B", &["A"]).is_err());
    }
    #[test]
    fn parallel_insertion_order_preserves_relations() {
        let forward = sample();
        let mut reverse = Dag::new();
        reverse.insert("A", &[]).unwrap();
        reverse.insert("B", &["A"]).unwrap();
        reverse.insert("D", &["B"]).unwrap();
        reverse.insert("C", &["B"]).unwrap();
        reverse.insert("E", &["D", "C"]).unwrap();
        for id in ["A", "B", "C", "D", "E"] {
            assert_eq!(forward.past(id).unwrap(), reverse.past(id).unwrap());
            assert_eq!(forward.anticone(id).unwrap(), reverse.anticone(id).unwrap());
        }
    }
}
