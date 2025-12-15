use crate::traits::*;
use serde::{Deserialize, Serialize};

// Define a User definition with User model
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserTreeNames {
    Users,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UserKeys {
    Users(UserId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserDefinition {
    Users(User),
}

// Define a Post definition with Post model that references User
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PostId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Post {
    pub id: PostId,
    pub title: String,
    pub content: String,
    pub author_id: UserId, // Foreign key to User
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PostTreeNames {
    Posts,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PostKeys {
    Posts(PostId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostDefinition {
    Posts(Post),
}

// Implement traits for User
impl NetabaseModelMarker for User {
    type Definition = UserDefinition;
    type Keys = UserKeys;
}

impl NetabaseDefinition for UserDefinition {
    type TreeNames = UserTreeNames;
    type Keys = UserKeys;

    fn tree_name(&self) -> Self::TreeNames {
        match self {
            Self::Users(_) => UserTreeNames::Users,
        }
    }

    fn key(&self) -> Self::Keys {
        match self {
            Self::Users(user) => UserKeys::Users(user.id.clone()),
        }
    }
}

impl From<User> for UserDefinition {
    fn from(user: User) -> Self {
        Self::Users(user)
    }
}

impl TryFrom<UserDefinition> for User {
    type Error = ();

    fn try_from(def: UserDefinition) -> Result<Self, Self::Error> {
        match def {
            UserDefinition::Users(user) => Ok(user),
        }
    }
}

impl NetabaseModel<UserDefinition> for User {
    type PrimaryKey = UserId;

    fn primary_key(&self) -> Self::PrimaryKey {
        self.id.clone()
    }
}

// Implement traits for Post
impl NetabaseModelMarker for Post {
    type Definition = PostDefinition;
    type Keys = PostKeys;
}

impl NetabaseDefinition for PostDefinition {
    type TreeNames = PostTreeNames;
    type Keys = PostKeys;

    fn tree_name(&self) -> Self::TreeNames {
        match self {
            Self::Posts(_) => PostTreeNames::Posts,
        }
    }

    fn key(&self) -> Self::Keys {
        match self {
            Self::Posts(post) => PostKeys::Posts(post.id.clone()),
        }
    }
}

impl From<Post> for PostDefinition {
    fn from(post: Post) -> Self {
        Self::Posts(post)
    }
}

impl TryFrom<PostDefinition> for Post {
    type Error = ();

    fn try_from(def: PostDefinition) -> Result<Self, Self::Error> {
        match def {
            PostDefinition::Posts(post) => Ok(post),
        }
    }
}

impl NetabaseModel<PostDefinition> for Post {
    type PrimaryKey = PostId;

    fn primary_key(&self) -> Self::PrimaryKey {
        self.id.clone()
    }
}

// Define a relational key for Post -> User relationship
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostAuthorRelation {
    pub post_id: PostId,
    pub user_id: UserId,
}

impl NetabaseRelationalKeyMarker for PostAuthorRelation {
    type SourceDefinition = PostDefinition;
    type SourceModel = Post;
    type SourceKey = PostId;
    type ForeignDefinition = UserDefinition;
    type ForeignModel = User;
    type ForeignKey = UserId;
}

impl NetabaseRelationalKey<PostDefinition, Post, PostId> for PostAuthorRelation {
    fn source_key(&self) -> PostId {
        self.post_id.clone()
    }
}

impl NetabaseModelRelationalKeyMarker<PostDefinition, Post, PostId, UserDefinition, User, UserId> for PostAuthorRelation {
    fn foreign_key(&self) -> UserId {
        self.user_id.clone()
    }
}

// Create global definition enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalDefinition {
    User(UserDefinition),
    Post(PostDefinition),
}

impl From<UserDefinition> for GlobalDefinition {
    fn from(def: UserDefinition) -> Self {
        Self::User(def)
    }
}

impl From<PostDefinition> for GlobalDefinition {
    fn from(def: PostDefinition) -> Self {
        Self::Post(def)
    }
}

impl TryFrom<GlobalDefinition> for UserDefinition {
    type Error = ();

    fn try_from(global: GlobalDefinition) -> Result<Self, Self::Error> {
        match global {
            GlobalDefinition::User(def) => Ok(def),
            _ => Err(()),
        }
    }
}

impl TryFrom<GlobalDefinition> for PostDefinition {
    type Error = ();

    fn try_from(global: GlobalDefinition) -> Result<Self, Self::Error> {
        match global {
            GlobalDefinition::Post(def) => Ok(def),
            _ => Err(()),
        }
    }
}

// Test function demonstrating cross-definition access
pub fn test_cross_definition_access() {
    // Create test data
    let user = User {
        id: UserId(1),
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };

    let post = Post {
        id: PostId(100),
        title: "My First Post".to_string(),
        content: "This is my first blog post!".to_string(),
        author_id: user.id.clone(),
    };

    // Create relational link
    let relation = PostAuthorRelation {
        post_id: post.id.clone(),
        user_id: post.author_id.clone(),
    };

    // Convert to definitions
    let user_def = UserDefinition::from(user.clone());
    let post_def = PostDefinition::from(post.clone());

    // Convert to global definitions
    let global_user = GlobalDefinition::from(user_def.clone());
    let global_post = GlobalDefinition::from(post_def.clone());

    // Test conversions back
    let recovered_user_def = UserDefinition::try_from(global_user).unwrap();
    let recovered_post_def = PostDefinition::try_from(global_post).unwrap();

    assert_eq!(user_def, recovered_user_def);
    assert_eq!(post_def, recovered_post_def);

    // Test relational key functionality
    assert_eq!(relation.source_key(), post.id);
    assert_eq!(relation.foreign_key(), user.id);

    println!("Cross-definition access test passed!");
    println!("User: {:?}", user);
    println!("Post: {:?}", post);
    println!("Relation: {:?}", relation);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_definition_relationships() {
        test_cross_definition_access();
    }

    #[test]
    fn test_relational_keys() {
        let relation = PostAuthorRelation {
            post_id: PostId(1),
            user_id: UserId(42),
        };

        assert_eq!(relation.source_key(), PostId(1));
        assert_eq!(relation.foreign_key(), UserId(42));
    }

    #[test]
    fn test_global_definition_conversions() {
        let user = User {
            id: UserId(1),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };

        let user_def = UserDefinition::from(user);
        let global_def = GlobalDefinition::from(user_def.clone());
        let recovered_def = UserDefinition::try_from(global_def).unwrap();

        assert_eq!(user_def, recovered_def);
    }
}