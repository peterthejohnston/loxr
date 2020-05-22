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
    entries: Vec<Slot>,
    count: usize,
}

#[derive(Clone)]
enum Slot {
    Empty,
    Tombstone, // Marks deleted entries
    Entry(Entry),
}

#[derive(Clone)]
struct Entry {
    key: String,
    value: Value,
}

impl Default for Table {
    fn default() -> Self {
        Table {
            entries: vec![],
            count: 0,
        }
    }
}

impl Table {
    fn grow_capacity(cap: usize) -> usize {
        if cap < 8 {
            8
        } else {
            cap * 2
        }
    }

    fn adjust_capacity(&mut self, cap: usize) {
        let mut new_entries = vec![Slot::Empty; cap];
        self.count = 0;
        for entry in &self.entries {
            if let Slot::Entry(entry) = entry {
                let (i, _) = self.find_entry(&new_entries, &entry.key);
                new_entries[i] = Slot::Entry(entry.clone());
                self.count += 1;
            }
        }
        self.entries = new_entries;
    }

    fn find_entry<'a>(&self, entries: &'a Vec<Slot>, key: &str) -> (usize, &'a Slot) {
        let mut i = hash(key) % entries.len();
        let mut last_tombstone: Option<&Slot> = None;
        loop {
            let slot = &entries[i];
            match slot {
                Slot::Tombstone => last_tombstone = Some(slot),
                Slot::Empty => match last_tombstone {
                    None => return (i, &Slot::Empty),
                    Some(tombstone) => return (i, tombstone),
                },
                Slot::Entry(entry) => {
                    if entry.key == key {
                        return (i, &slot);
                    }
                }
            }
            i = (i + 1) % entries.len();
        }
    }

    pub fn insert(&mut self, key: &str, value: Value) -> Option<Value> {
        if (self.count + 1) as f64 > self.entries.len() as f64 * MAX_LOAD {
            let new_cap = Self::grow_capacity(self.entries.len());
            self.adjust_capacity(new_cap);
        }

        let (i, entry) = self.find_entry(&self.entries, &key);
        let result = match entry {
            Slot::Entry(entry) => Some(entry.value.clone()),
            Slot::Tombstone => None,
            Slot::Empty => {
                self.count += 1;
                None
            }
        };
        self.entries[i] = Slot::Entry(Entry {
            key: key.into(),
            value,
        });
        result
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        if self.count == 0 {
            return None;
        };

        match self.find_entry(&self.entries, key) {
            (_, Slot::Entry(entry)) => Some(&entry.value),
            _ => None,
        }
    }

    fn delete(&mut self, key: &str) -> bool {
        if self.count == 0 {
            return false;
        }

        let (i, entry) = self.find_entry(&self.entries, key);
        match entry {
            Slot::Entry(_) => {
                // Place a tombstone in the entry.
                self.entries[i] = Slot::Tombstone;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::table::Table;
    use crate::value::Value;

    #[test]
    fn lots_of_entries() {
        let mut table = Table::default();
        for i in 0..100 {
            assert_eq!(table.insert(&i.to_string(), Value::Number(i as f64)), None);
        }
        for i in 0..100 {
            assert_eq!(table.get(&i.to_string()), Some(&Value::Number(i as f64)));
        }
        for i in -100..0 {
            assert_eq!(table.get(&i.to_string()), None);
        }
        for i in 101..200 {
            assert_eq!(table.get(&i.to_string()), None);
        }
    }

    #[test]
    fn delete_entries() {
        let mut table = Table::default();
        for i in 0..100 {
            assert_eq!(table.insert(&i.to_string(), Value::Number(i as f64)), None);
        }
        for i in (0..100).skip(1).step_by(2) {
            assert_eq!(table.delete(&i.to_string()), true);
        }
        for i in (0..100).step_by(2) {
            assert_eq!(table.get(&i.to_string()), Some(&Value::Number(i as f64)));
        }
        for i in (0..100).skip(1).step_by(2) {
            assert_eq!(table.get(&i.to_string()), None);
        }
    }
}
