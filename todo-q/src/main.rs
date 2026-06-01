use borsh::{BorshDeserialize, BorshSerialize};
use std::{collections::VecDeque, error::Error, time::UNIX_EPOCH};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Todo {
    pub id: u64,
    pub description: String,
    pub created_at: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Queue<T> {
    pub items: VecDeque<T>,
}

impl<T> Queue<T> {
    pub fn enqueue(&mut self, item: T) {
        self.items.push_back(item)
    }

    pub fn dequeue(&mut self) -> Option<T> {
        self.items.pop_front()
    }

    pub fn peek(&self) -> Option<&T> {
        self.items.front()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T> Queue<T>
where
    T: BorshSerialize + BorshDeserialize,
{
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let bytes = borsh::to_vec(self)?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(borsh::from_slice(&bytes)?),
            Err(_) => Ok(Self {
                items: VecDeque::new(),
            }),
        }
    }
}

impl Queue<Todo> {
    pub fn add_todo(&mut self, description: String, path: &str) -> Result<(), Box<dyn Error>> {
        let next_id = self.items.back().map_or(1, |todo| todo.id + 1);

        let created_at = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let new_todo = Todo {
            id: next_id,
            description,
            created_at,
        };

        self.enqueue(new_todo);
        self.save_to_file(path)?;
        println!("Todo added! \nID: {}", next_id);
        Ok(())
    }

    pub fn list_tasks(&self) {
        if self.is_empty() {
            println!("Queue is empty. No tasks!");
            return;
        }
        println!("....................................................");
        println!("::::::::::::         Todos          ::::::::::::::::");
        println!("....................................................\n");
        println!("Total todos: {}\n", self.len());
        for todo in &self.items {
            println!(
                "Todo ({})\t {} [created at: {}]",
                todo.id, todo.description, todo.created_at
            )
        }
    }

    pub fn complete_next(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        match self.dequeue() {
            Some(todo) => {
                self.save_to_file(path)?;
                println!("Completed Task: `{}`", todo.description)
            }
            None => {
                println!("Queue is empty. No task to complete!")
            }
        }
        Ok(())
    }

    pub fn see_latest(&self) {
        if self.is_empty() {
            println!("No tasks to display.");
            return;
        }
        let todo = self.items.back().unwrap();
        println!(
            "Todo ({})\t {}  [created at: {}]",
            todo.id, todo.description, todo.created_at
        )
    }
}

fn main() {
    let path = "todos.bin";
    let args: Vec<String> = std::env::args().collect();
    let mut queue = Queue::<Todo>::load_from_file(path).unwrap();

    match args.get(1).map(String::as_str) {
        Some("add") => {
            let description = args[2..].join(" ");
            queue.add_todo(description, path).unwrap();
        }

        Some("list") => {
            queue.list_tasks();
        }

        Some("done") => {
            queue.complete_next(path).unwrap();
        }

        Some("latest") => {
            queue.see_latest();
        }

        Some("help") => {
            println!("[add] [task to add] => To add new todo");
            println!("[list] => To list all todos");
            println!("[done] => To mark oldest task as done.");
            println!("[latest] => To view latest todo.");
            println!("[help] => To view available command.")
        }

        _ => {
            println!("Invalid arg: {:?} ...", args.get(1));
            println!("[help] => To view available command.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let path = "test_todos.bin";
        let mut queue = Queue::<Todo>::load_from_file(path).unwrap();
        let description_txt = "Test add todo".to_string();
        queue.add_todo(description_txt, path).unwrap();
        let todo = queue.peek().unwrap();
        assert_eq!(todo.description, "Test add todo".to_string());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_serialize() {
        let path = "test_serialize_todos.bin";
        let mut queue = Queue::<Todo>::load_from_file(path).unwrap();
        queue.add_todo("Test serialize".to_string(), path).unwrap();
        let bytes = std::fs::read(path).unwrap();
        assert!(bytes.len() > 0);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_deserialize() {
        let path = "test_deserialize_todos.bin";
        let mut queue = Queue::<Todo>::load_from_file(path).unwrap();
        queue
            .add_todo("Test deserialize".to_string(), path)
            .unwrap();

        // Load fresh from disk
        let loaded = Queue::<Todo>::load_from_file(path).unwrap();
        assert!(loaded.len() > 0);
        assert_eq!(loaded.peek().unwrap().description, "Test deserialize");
        let _ = std::fs::remove_file(path);
    }
}
