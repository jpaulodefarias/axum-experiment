use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(name: String, email: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            email,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update(&mut self, name: String, email: String) {
        self.name = name;
        self.email = email;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_changes_fields_and_timestamp() {
        let mut user = User::new("Before".into(), "before@example.com".into());
        let previous = user.updated_at;
        user.update("After".into(), "after@example.com".into());
        assert_eq!(user.name, "After");
        assert_eq!(user.email, "after@example.com");
        assert!(user.updated_at >= previous);
    }
}
