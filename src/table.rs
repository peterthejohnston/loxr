use crate::value::Value;

const MAX_LOAD: f64 = 0.75;

fn hash(s: &str) -> usize {
    let mut hash: usize = 2166136261;
    for c in s.chars() {
      hash ^= c as usize;
      hash = hash.wrapping_mul(16777619);
    }
    hash
}

pub struct Table {
    entries: Vec<Option<Entry>>,
    count: usize,
}

#[derive(Clone)]
struct Entry {
    key: String,
    value: Value,
}

impl Table {
    pub fn new() -> Table {
        Table {
            entries: vec![],
            count: 0,
        }
    }

    fn grow_capacity(cap: usize) -> usize {
        if cap < 8 { 8 } else { cap * 2 }
    }

    fn adjust_capacity(&mut self, cap: usize) {
        let mut new_entries = vec![None; cap];
        for entry in &self.entries {
            if let Some(entry) = entry {
                let (i, _) = self.find_entry(&new_entries, &entry.key);
                new_entries[i] = Some(entry.clone());
            }
        }
        self.entries = new_entries;
    }

    fn find_entry<'a>(&self, entries: &'a Vec<Option<Entry>>, key: &str)
    -> (usize, Option<&'a Entry>)
    {
        let mut i = hash(key) % entries.len();
        loop {
            let entry = &entries[i];
            if entry.is_none() { return (i, None); }
            if let Some(entry) = entry {
                if entry.key == key {
                    return (i, Some(entry));
                }
            }
            i = (i + 1) % entries.len();
        }
    }

    pub fn insert(&mut self, key: &str, value: Value) -> bool {
        if (self.count + 1) as f64 > self.entries.len() as f64 * MAX_LOAD {
            let new_cap = Self::grow_capacity(self.entries.len());
            self.adjust_capacity(new_cap);
        }

        let (i, entry) = self.find_entry(&self.entries, &key);
        let is_new_key = entry.is_none();
        if is_new_key {
            self.count += 1;
        }
        self.entries[i] = Some(Entry { key: key.into(), value });
        is_new_key
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        if self.count == 0 { return None };

        match self.find_entry(&self.entries, key) {
            (_, None) => None,
            (_, Some(entry)) => Some(entry.value.clone()),
        }
    }

    fn delete(&self, key: &str) {
        // TODO
    }
}

#[cfg(test)]
mod tests {
    use crate::table::Table;
    use crate::value::Value;

    #[test]
    fn lots_of_entries() {
        let mut table = Table::new();
        for i in 0..100 {
            table.insert(&i.to_string(), Value::Number(i as f64));
        }
        for i in 0..100 {
            assert_eq!(table.get(&i.to_string()), Some(Value::Number(i as f64)));
        }
        for i in -100..0 {
            assert_eq!(table.get(&i.to_string()), None);
        }
        for i in 101..200 {
            assert_eq!(table.get(&i.to_string()), None);
        }
    }
}
