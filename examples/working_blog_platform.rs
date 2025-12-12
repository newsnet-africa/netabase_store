use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete Blog Platform Example
/// 
/// Demonstrates:
/// - Hierarchical content organization
/// - User roles and permissions
/// - Content moderation workflows
/// - Comment threading
/// - Tag-based categorization

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    Editor,
    Author,
    Subscriber,
    Moderator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PostStatus {
    Draft,
    Pending,
    Published,
    Archived,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CommentStatus {
    Pending,
    Approved,
    Rejected,
    Flagged,
}

/// User with role-based permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub role: UserRole,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: u64,
    pub last_active: u64,
    pub is_active: bool,
}

/// Blog post with hierarchical organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Post {
    pub id: u64,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: String,
    pub author_id: u64,
    pub category_id: u64,
    pub tags: Vec<String>,
    pub status: PostStatus,
    pub view_count: u64,
    pub featured_image: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    pub published_at: Option<u64>,
}

/// Content categories for hierarchical organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    pub id: u64,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub parent_id: Option<u64>,
    pub sort_order: u32,
    pub is_active: bool,
    pub created_at: u64,
}

/// Comments with threading support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Comment {
    pub id: u64,
    pub post_id: u64,
    pub author_id: u64,
    pub parent_id: Option<u64>, // For threaded comments
    pub content: String,
    pub status: CommentStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub likes: u32,
    pub dislikes: u32,
}

/// Tags for flexible categorization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    pub id: u64,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: String,
    pub post_count: u64,
    pub created_at: u64,
}

/// In-memory blog platform for demonstration
pub struct BlogPlatform {
    pub users: HashMap<u64, User>,
    pub posts: HashMap<u64, Post>,
    pub categories: HashMap<u64, Category>,
    pub comments: HashMap<u64, Comment>,
    pub tags: HashMap<u64, Tag>,
    next_id: u64,
}

impl BlogPlatform {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(BlogPlatform {
            users: HashMap::new(),
            posts: HashMap::new(),
            categories: HashMap::new(),
            comments: HashMap::new(),
            tags: HashMap::new(),
            next_id: 1,
        })
    }

    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Create a new user
    pub fn create_user(&mut self, mut user: User) -> Result<u64, Box<dyn std::error::Error>> {
        if user.username.len() < 3 {
            return Err("Username must be at least 3 characters".into());
        }

        // Check for existing username/email
        for existing_user in self.users.values() {
            if existing_user.username == user.username {
                return Err("Username already exists".into());
            }
            if existing_user.email == user.email {
                return Err("Email already exists".into());
            }
        }

        user.id = self.next_id();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        user.created_at = now;
        user.last_active = now;

        let user_id = user.id;
        self.users.insert(user_id, user);
        Ok(user_id)
    }

    /// Create a new category
    pub fn create_category(&mut self, mut category: Category) -> Result<u64, Box<dyn std::error::Error>> {
        if category.name.is_empty() {
            return Err("Category name cannot be empty".into());
        }

        // Check for existing slug
        for existing_category in self.categories.values() {
            if existing_category.slug == category.slug {
                return Err("Category slug already exists".into());
            }
        }

        // Validate parent category exists if specified
        if let Some(parent_id) = category.parent_id {
            if !self.categories.contains_key(&parent_id) {
                return Err("Parent category does not exist".into());
            }
        }

        category.id = self.next_id();
        category.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let category_id = category.id;
        self.categories.insert(category_id, category);
        Ok(category_id)
    }

    /// Create a new blog post
    pub fn create_post(&mut self, mut post: Post) -> Result<u64, Box<dyn std::error::Error>> {
        if post.title.is_empty() {
            return Err("Post title cannot be empty".into());
        }

        // Validate author exists
        if !self.users.contains_key(&post.author_id) {
            return Err("Author does not exist".into());
        }

        // Validate category exists
        if !self.categories.contains_key(&post.category_id) {
            return Err("Category does not exist".into());
        }

        // Check for existing slug
        for existing_post in self.posts.values() {
            if existing_post.slug == post.slug {
                return Err("Post slug already exists".into());
            }
        }

        post.id = self.next_id();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        post.created_at = now;
        post.updated_at = now;

        if matches!(post.status, PostStatus::Published) && post.published_at.is_none() {
            post.published_at = Some(now);
        }

        let post_id = post.id;
        self.posts.insert(post_id, post);
        Ok(post_id)
    }

    /// Add a comment to a post
    pub fn add_comment(&mut self, mut comment: Comment) -> Result<u64, Box<dyn std::error::Error>> {
        // Validate post exists
        if !self.posts.contains_key(&comment.post_id) {
            return Err("Post does not exist".into());
        }

        // Validate author exists
        if !self.users.contains_key(&comment.author_id) {
            return Err("Comment author does not exist".into());
        }

        // Validate parent comment exists if specified
        if let Some(parent_id) = comment.parent_id {
            if let Some(parent_comment) = self.comments.get(&parent_id) {
                // Ensure parent comment is on the same post
                if parent_comment.post_id != comment.post_id {
                    return Err("Parent comment is not on the same post".into());
                }
            } else {
                return Err("Parent comment does not exist".into());
            }
        }

        comment.id = self.next_id();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        comment.created_at = now;
        comment.updated_at = now;

        let comment_id = comment.id;
        self.comments.insert(comment_id, comment);
        Ok(comment_id)
    }

    /// Create or get a tag
    pub fn create_tag(&mut self, name: String) -> Result<u64, Box<dyn std::error::Error>> {
        let slug = name.to_lowercase().replace(' ', "-");
        
        // Check if tag already exists
        for tag in self.tags.values() {
            if tag.slug == slug {
                return Ok(tag.id);
            }
        }

        let tag = Tag {
            id: self.next_id(),
            name: name.clone(),
            slug,
            description: None,
            color: "#3498db".to_string(),
            post_count: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let tag_id = tag.id;
        self.tags.insert(tag_id, tag);
        Ok(tag_id)
    }

    /// Publish a draft post
    pub fn publish_post(&mut self, post_id: u64, publisher_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        let post = self.posts.get_mut(&post_id).ok_or("Post not found")?;
        let publisher = self.users.get(&publisher_id).ok_or("Publisher not found")?;

        // Check permissions
        match publisher.role {
            UserRole::Admin | UserRole::Editor => {
                // Can publish any post
            }
            UserRole::Author => {
                // Can only publish their own posts
                if post.author_id != publisher_id {
                    return Err("Authors can only publish their own posts".into());
                }
            }
            _ => return Err("Insufficient permissions to publish posts".into()),
        }

        post.status = PostStatus::Published;
        post.published_at = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs());
        post.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }

    /// Moderate a comment
    pub fn moderate_comment(&mut self, comment_id: u64, moderator_id: u64, new_status: CommentStatus) -> Result<(), Box<dyn std::error::Error>> {
        let comment = self.comments.get_mut(&comment_id).ok_or("Comment not found")?;
        let moderator = self.users.get(&moderator_id).ok_or("Moderator not found")?;

        // Check permissions
        match moderator.role {
            UserRole::Admin | UserRole::Moderator | UserRole::Editor => {
                // Can moderate comments
            }
            _ => return Err("Insufficient permissions to moderate comments".into()),
        }

        comment.status = new_status;
        comment.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }

    /// Get posts by category (with hierarchy support)
    pub fn get_posts_by_category(&self, category_id: u64, include_subcategories: bool) -> Vec<&Post> {
        let mut category_ids = vec![category_id];
        
        if include_subcategories {
            // Find all subcategories
            let subcategories: Vec<_> = self.categories.values()
                .filter(|cat| cat.parent_id == Some(category_id))
                .map(|cat| cat.id)
                .collect();
            category_ids.extend(subcategories);
        }

        self.posts.values()
            .filter(|post| {
                matches!(post.status, PostStatus::Published) && 
                category_ids.contains(&post.category_id)
            })
            .collect()
    }

    /// Get posts by tag
    pub fn get_posts_by_tag(&self, tag_name: &str) -> Vec<&Post> {
        self.posts.values()
            .filter(|post| {
                matches!(post.status, PostStatus::Published) && 
                post.tags.iter().any(|tag| tag.to_lowercase() == tag_name.to_lowercase())
            })
            .collect()
    }

    /// Get comments for a post (threaded)
    pub fn get_post_comments(&self, post_id: u64) -> Vec<&Comment> {
        self.comments.values()
            .filter(|comment| {
                comment.post_id == post_id && 
                matches!(comment.status, CommentStatus::Approved)
            })
            .collect()
    }

    /// Search posts
    pub fn search_posts(&self, query: &str) -> Vec<&Post> {
        let term = query.to_lowercase();
        self.posts.values()
            .filter(|post| {
                matches!(post.status, PostStatus::Published) && (
                    post.title.to_lowercase().contains(&term) ||
                    post.content.to_lowercase().contains(&term) ||
                    post.excerpt.to_lowercase().contains(&term) ||
                    post.tags.iter().any(|tag| tag.to_lowercase().contains(&term))
                )
            })
            .collect()
    }

    /// Get user's posts
    pub fn get_user_posts(&self, user_id: u64) -> Vec<&Post> {
        self.posts.values()
            .filter(|post| post.author_id == user_id)
            .collect()
    }

    /// Increment post view count
    pub fn increment_view_count(&mut self, post_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(post) = self.posts.get_mut(&post_id) {
            post.view_count += 1;
            Ok(())
        } else {
            Err("Post not found".into())
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Netabase Blog Platform Demo");
    println!("==============================");

    let mut blog = BlogPlatform::new()?;

    // Create users with different roles
    let admin = User {
        id: 0,
        username: "admin".to_string(),
        email: "admin@blog.com".to_string(),
        display_name: "Site Administrator".to_string(),
        role: UserRole::Admin,
        bio: Some("Managing the blog platform".to_string()),
        avatar_url: None,
        created_at: 0,
        last_active: 0,
        is_active: true,
    };

    let author = User {
        id: 0,
        username: "alice_writer".to_string(),
        email: "alice@example.com".to_string(),
        display_name: "Alice Johnson".to_string(),
        role: UserRole::Author,
        bio: Some("Tech writer and blogger".to_string()),
        avatar_url: None,
        created_at: 0,
        last_active: 0,
        is_active: true,
    };

    let subscriber = User {
        id: 0,
        username: "reader_bob".to_string(),
        email: "bob@example.com".to_string(),
        display_name: "Bob Reader".to_string(),
        role: UserRole::Subscriber,
        bio: None,
        avatar_url: None,
        created_at: 0,
        last_active: 0,
        is_active: true,
    };

    let admin_id = blog.create_user(admin)?;
    let author_id = blog.create_user(author)?;
    let subscriber_id = blog.create_user(subscriber)?;

    println!("✅ Created users: Admin ({}), Author ({}), Subscriber ({})", admin_id, author_id, subscriber_id);

    // Create categories with hierarchy
    let tech_category = Category {
        id: 0,
        name: "Technology".to_string(),
        slug: "technology".to_string(),
        description: "Technology-related posts".to_string(),
        parent_id: None,
        sort_order: 1,
        is_active: true,
        created_at: 0,
    };

    let rust_category = Category {
        id: 0,
        name: "Rust Programming".to_string(),
        slug: "rust-programming".to_string(),
        description: "Posts about Rust programming language".to_string(),
        parent_id: None, // Will be updated after tech category is created
        sort_order: 1,
        is_active: true,
        created_at: 0,
    };

    let tech_id = blog.create_category(tech_category)?;
    
    let mut rust_category_updated = rust_category;
    rust_category_updated.parent_id = Some(tech_id);
    let rust_id = blog.create_category(rust_category_updated)?;

    println!("✅ Created categories: Technology ({}) and Rust Programming ({})", tech_id, rust_id);

    // Create blog posts
    let post1 = Post {
        id: 0,
        title: "Getting Started with Rust".to_string(),
        slug: "getting-started-with-rust".to_string(),
        content: "Rust is a systems programming language that is fast, memory-safe, and thread-safe...".to_string(),
        excerpt: "Learn the basics of Rust programming language".to_string(),
        author_id,
        category_id: rust_id,
        tags: vec!["rust".to_string(), "programming".to_string(), "tutorial".to_string()],
        status: PostStatus::Draft,
        view_count: 0,
        featured_image: Some("rust_tutorial.jpg".to_string()),
        created_at: 0,
        updated_at: 0,
        published_at: None,
    };

    let post2 = Post {
        id: 0,
        title: "Database Design Patterns".to_string(),
        slug: "database-design-patterns".to_string(),
        content: "Effective database design is crucial for application performance...".to_string(),
        excerpt: "Learn about common database design patterns and best practices".to_string(),
        author_id,
        category_id: tech_id,
        tags: vec!["database".to_string(), "design".to_string(), "architecture".to_string()],
        status: PostStatus::Draft,
        view_count: 0,
        featured_image: None,
        created_at: 0,
        updated_at: 0,
        published_at: None,
    };

    let post1_id = blog.create_post(post1)?;
    let post2_id = blog.create_post(post2)?;

    println!("✅ Created posts: Rust Tutorial ({}) and Database Patterns ({})", post1_id, post2_id);

    // Publish the posts
    blog.publish_post(post1_id, admin_id)?;
    blog.publish_post(post2_id, author_id)?; // Author publishes their own post

    println!("✅ Published posts");

    // Add comments with threading
    let comment1 = Comment {
        id: 0,
        post_id: post1_id,
        author_id: subscriber_id,
        parent_id: None,
        content: "Great tutorial! Very helpful for beginners.".to_string(),
        status: CommentStatus::Approved,
        created_at: 0,
        updated_at: 0,
        likes: 0,
        dislikes: 0,
    };

    let comment1_id = blog.add_comment(comment1)?;

    let reply_comment = Comment {
        id: 0,
        post_id: post1_id,
        author_id: author_id,
        parent_id: Some(comment1_id),
        content: "Thank you! I'm glad you found it helpful.".to_string(),
        status: CommentStatus::Approved,
        created_at: 0,
        updated_at: 0,
        likes: 0,
        dislikes: 0,
    };

    let reply_id = blog.add_comment(reply_comment)?;

    println!("✅ Added comments with threading: Comment ({}) and Reply ({})", comment1_id, reply_id);

    // Demonstrate search functionality
    let search_results = blog.search_posts("rust");
    println!("🔍 Search results for 'rust': {} posts found", search_results.len());
    for post in search_results {
        println!("   - {} (Category: {})", post.title, post.category_id);
    }

    // Show posts by category
    let tech_posts = blog.get_posts_by_category(tech_id, true);
    println!("📁 Technology posts (including subcategories): {} posts", tech_posts.len());
    for post in tech_posts {
        println!("   - {} (Views: {})", post.title, post.view_count);
    }

    // Show user's posts
    let author_posts = blog.get_user_posts(author_id);
    println!("✍️ Author's posts: {} posts", author_posts.len());
    for post in author_posts {
        println!("   - {} ({:?})", post.title, post.status);
    }

    // Show post comments
    let post_comments = blog.get_post_comments(post1_id);
    println!("💬 Comments on Rust tutorial: {} comments", post_comments.len());
    for comment in post_comments {
        let indent = if comment.parent_id.is_some() { "    " } else { "" };
        println!("   {}Comment by user {}: {}", indent, comment.author_id, comment.content);
    }

    // Increment view counts
    blog.increment_view_count(post1_id)?;
    blog.increment_view_count(post1_id)?;
    blog.increment_view_count(post2_id)?;

    println!("👁️ Updated view counts");

    println!("\n🎉 Blog platform demo completed successfully!");
    
    Ok(())
}