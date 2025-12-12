use netabase_store::prelude::*;
use netabase_macros::netabase;
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
    Spam,
}

/// User with hierarchical permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[netabase(permissions = "hierarchical")]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub role: UserRole,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: u64,
    pub last_login: Option<u64>,
    pub is_active: bool,
    pub post_count: u64,
    pub comment_count: u64,
}

/// Category for organizing posts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[netabase(permissions = "hierarchical", cross_links = ["Post"])]
pub struct Category {
    pub id: u64,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub parent_id: Option<u64>,
    pub post_count: u64,
    pub created_at: u64,
    pub is_active: bool,
}

/// Tag for categorizing content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[netabase(cross_links = ["Post"])]
pub struct Tag {
    pub id: u64,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub usage_count: u64,
    pub created_at: u64,
}

/// Blog post with rich content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[netabase(cross_links = ["User", "Category", "Tag", "Comment"], relationships = ["User:ManyToOne", "Category:ManyToOne"])]
pub struct Post {
    pub id: u64,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub excerpt: String,
    pub author_id: u64,
    pub category_id: u64,
    pub tag_ids: Vec<u64>,
    pub status: PostStatus,
    pub featured_image: Option<String>,
    pub meta_description: Option<String>,
    pub view_count: u64,
    pub like_count: u64,
    pub comment_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub published_at: Option<u64>,
    pub scheduled_at: Option<u64>,
}

/// Comment with threading support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[netabase(relationships = ["User:ManyToOne", "Post:ManyToOne"])]
pub struct Comment {
    pub id: u64,
    pub post_id: u64,
    pub user_id: u64,
    pub parent_id: Option<u64>, // For threading
    pub content: String,
    pub status: CommentStatus,
    pub like_count: u64,
    pub reply_count: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub ip_address: String,
    pub user_agent: String,
}

/// Media file management
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[netabase(relationships = ["User:ManyToOne"])]
pub struct MediaFile {
    pub id: u64,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub file_size: u64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub upload_path: String,
    pub uploaded_by: u64,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
    pub created_at: u64,
}

/// Newsletter subscription
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[netabase]
pub struct Newsletter {
    pub id: u64,
    pub email: String,
    pub name: Option<String>,
    pub subscribed_at: u64,
    pub confirmed_at: Option<u64>,
    pub unsubscribed_at: Option<u64>,
    pub is_active: bool,
    pub preferences: NewsletterPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewsletterPreferences {
    pub weekly_digest: bool,
    pub new_posts: bool,
    pub author_updates: bool,
    pub categories: Vec<u64>,
}

/// Blog platform application
pub struct BlogPlatform {
    pub user_store: UserStore<InMemoryBackend>,
    pub category_store: CategoryStore<InMemoryBackend>,
    pub tag_store: TagStore<InMemoryBackend>,
    pub post_store: PostStore<InMemoryBackend>,
    pub comment_store: CommentStore<InMemoryBackend>,
    pub media_store: MediaFileStore<InMemoryBackend>,
    pub newsletter_store: NewsletterStore<InMemoryBackend>,
}

impl BlogPlatform {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(BlogPlatform {
            user_store: UserStore::new_in_memory()?,
            category_store: CategoryStore::new_in_memory()?,
            tag_store: TagStore::new_in_memory()?,
            post_store: PostStore::new_in_memory()?,
            comment_store: CommentStore::new_in_memory()?,
            media_store: MediaFileStore::new_in_memory()?,
            newsletter_store: NewsletterStore::new_in_memory()?,
        })
    }
    
    /// Create a new user
    pub fn create_user(&mut self, user: User) -> Result<(), Box<dyn std::error::Error>> {
        // Validate user data
        if user.username.len() < 3 {
            return Err("Username must be at least 3 characters".into());
        }
        
        if !user.email.contains('@') {
            return Err("Invalid email format".into());
        }
        
        // Check for duplicates
        for existing_user in self.user_store.scan()? {
            if existing_user.username == user.username {
                return Err("Username already exists".into());
            }
            if existing_user.email == user.email {
                return Err("Email already registered".into());
            }
        }
        
        self.user_store.insert(user.id, user)?;
        Ok(())
    }
    
    /// Create a category
    pub fn create_category(&mut self, category: Category) -> Result<(), Box<dyn std::error::Error>> {
        // Validate category data
        if category.name.is_empty() {
            return Err("Category name cannot be empty".into());
        }
        
        // Check for duplicate slug
        for existing_category in self.category_store.scan()? {
            if existing_category.slug == category.slug {
                return Err("Category slug already exists".into());
            }
        }
        
        // If has parent, validate parent exists
        if let Some(parent_id) = category.parent_id {
            if self.category_store.get(&parent_id)?.is_none() {
                return Err("Parent category not found".into());
            }
        }
        
        self.category_store.insert(category.id, category)?;
        Ok(())
    }
    
    /// Create a tag
    pub fn create_tag(&mut self, tag: Tag) -> Result<(), Box<dyn std::error::Error>> {
        if tag.name.is_empty() {
            return Err("Tag name cannot be empty".into());
        }
        
        // Check for duplicate slug
        for existing_tag in self.tag_store.scan()? {
            if existing_tag.slug == tag.slug {
                return Err("Tag slug already exists".into());
            }
        }
        
        self.tag_store.insert(tag.id, tag)?;
        Ok(())
    }
    
    /// Create a blog post
    pub fn create_post(&mut self, post: Post) -> Result<(), Box<dyn std::error::Error>> {
        // Validate author exists
        if self.user_store.get(&post.author_id)?.is_none() {
            return Err("Author not found".into());
        }
        
        // Validate category exists
        if self.category_store.get(&post.category_id)?.is_none() {
            return Err("Category not found".into());
        }
        
        // Validate all tags exist
        for tag_id in &post.tag_ids {
            if self.tag_store.get(tag_id)?.is_none() {
                return Err(format!("Tag {} not found", tag_id).into());
            }
        }
        
        // Check for duplicate slug
        for existing_post in self.post_store.scan()? {
            if existing_post.slug == post.slug && existing_post.id != post.id {
                return Err("Post slug already exists".into());
            }
        }
        
        // Update category post count
        if let Some(mut category) = self.category_store.get(&post.category_id)? {
            category.post_count += 1;
            self.category_store.insert(category.id, category)?;
        }
        
        // Update tag usage counts
        for tag_id in &post.tag_ids {
            if let Some(mut tag) = self.tag_store.get(tag_id)? {
                tag.usage_count += 1;
                self.tag_store.insert(tag.id, tag)?;
            }
        }
        
        // Update author post count
        if let Some(mut author) = self.user_store.get(&post.author_id)? {
            author.post_count += 1;
            self.user_store.insert(author.id, author)?;
        }
        
        self.post_store.insert(post.id, post)?;
        Ok(())
    }
    
    /// Add a comment to a post
    pub fn add_comment(&mut self, comment: Comment) -> Result<(), Box<dyn std::error::Error>> {
        // Validate post exists
        if self.post_store.get(&comment.post_id)?.is_none() {
            return Err("Post not found".into());
        }
        
        // Validate user exists
        if self.user_store.get(&comment.user_id)?.is_none() {
            return Err("User not found".into());
        }
        
        // If has parent, validate parent comment exists and belongs to same post
        if let Some(parent_id) = comment.parent_id {
            if let Some(parent_comment) = self.comment_store.get(&parent_id)? {
                if parent_comment.post_id != comment.post_id {
                    return Err("Parent comment belongs to different post".into());
                }
                
                // Update parent comment reply count
                let mut parent = parent_comment;
                parent.reply_count += 1;
                self.comment_store.insert(parent.id, parent)?;
            } else {
                return Err("Parent comment not found".into());
            }
        }
        
        // Update post comment count
        if let Some(mut post) = self.post_store.get(&comment.post_id)? {
            post.comment_count += 1;
            self.post_store.insert(post.id, post)?;
        }
        
        // Update user comment count
        if let Some(mut user) = self.user_store.get(&comment.user_id)? {
            user.comment_count += 1;
            self.user_store.insert(user.id, user)?;
        }
        
        self.comment_store.insert(comment.id, comment)?;
        Ok(())
    }
    
    /// Publish a post (editor permission required)
    pub fn publish_post(&mut self, post_id: u64, publisher_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        // Validate publisher has permission
        if let Some(publisher) = self.user_store.get(&publisher_id)? {
            if !matches!(publisher.role, UserRole::Admin | UserRole::Editor) {
                return Err("User does not have permission to publish posts".into());
            }
        } else {
            return Err("Publisher not found".into());
        }
        
        // Get and update post
        if let Some(mut post) = self.post_store.get(&post_id)? {
            post.status = PostStatus::Published;
            post.published_at = Some(std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs());
            post.updated_at = post.published_at.unwrap();
            
            self.post_store.insert(post.id, post)?;
        } else {
            return Err("Post not found".into());
        }
        
        Ok(())
    }
    
    /// Moderate a comment (moderator permission required)
    pub fn moderate_comment(
        &mut self, 
        comment_id: u64, 
        moderator_id: u64, 
        new_status: CommentStatus
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Validate moderator has permission
        if let Some(moderator) = self.user_store.get(&moderator_id)? {
            if !matches!(moderator.role, UserRole::Admin | UserRole::Moderator | UserRole::Editor) {
                return Err("User does not have permission to moderate comments".into());
            }
        } else {
            return Err("Moderator not found".into());
        }
        
        // Get and update comment
        if let Some(mut comment) = self.comment_store.get(&comment_id)? {
            comment.status = new_status;
            comment.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            self.comment_store.insert(comment.id, comment)?;
        } else {
            return Err("Comment not found".into());
        }
        
        Ok(())
    }
    
    /// Get posts by category
    pub fn get_posts_by_category(&self, category_id: u64) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let posts: Vec<Post> = self.post_store
            .scan()?
            .filter(|post| post.category_id == category_id && post.status == PostStatus::Published)
            .collect();
        Ok(posts)
    }
    
    /// Get posts by tag
    pub fn get_posts_by_tag(&self, tag_id: u64) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let posts: Vec<Post> = self.post_store
            .scan()?
            .filter(|post| post.tag_ids.contains(&tag_id) && post.status == PostStatus::Published)
            .collect();
        Ok(posts)
    }
    
    /// Get posts by author
    pub fn get_posts_by_author(&self, author_id: u64) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let posts: Vec<Post> = self.post_store
            .scan()?
            .filter(|post| post.author_id == author_id && post.status == PostStatus::Published)
            .collect();
        Ok(posts)
    }
    
    /// Get comments for a post with threading
    pub fn get_post_comments(&self, post_id: u64) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
        let mut comments: Vec<Comment> = self.comment_store
            .scan()?
            .filter(|comment| {
                comment.post_id == post_id && 
                matches!(comment.status, CommentStatus::Approved)
            })
            .collect();
        
        // Sort by parent relationship and creation time
        comments.sort_by(|a, b| {
            match (a.parent_id, b.parent_id) {
                (None, None) => a.created_at.cmp(&b.created_at),
                (None, Some(_)) => std::cmp::Ordering::Less,
                (Some(_), None) => std::cmp::Ordering::Greater,
                (Some(a_parent), Some(b_parent)) => {
                    if a_parent == b_parent {
                        a.created_at.cmp(&b.created_at)
                    } else {
                        a_parent.cmp(&b_parent)
                    }
                }
            }
        });
        
        Ok(comments)
    }
    
    /// Search posts by content
    pub fn search_posts(&self, query: &str) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let query = query.to_lowercase();
        let posts: Vec<Post> = self.post_store
            .scan()?
            .filter(|post| {
                post.status == PostStatus::Published &&
                (post.title.to_lowercase().contains(&query) ||
                 post.content.to_lowercase().contains(&query) ||
                 post.excerpt.to_lowercase().contains(&query))
            })
            .collect();
        Ok(posts)
    }
    
    /// Get popular posts
    pub fn get_popular_posts(&self, limit: usize) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let mut posts: Vec<Post> = self.post_store
            .scan()?
            .filter(|post| post.status == PostStatus::Published)
            .collect();
        
        // Sort by view count + like count
        posts.sort_by(|a, b| {
            let a_score = a.view_count + a.like_count * 2; // Weight likes more
            let b_score = b.view_count + b.like_count * 2;
            b_score.cmp(&a_score)
        });
        
        Ok(posts.into_iter().take(limit).collect())
    }
    
    /// Get recent posts
    pub fn get_recent_posts(&self, limit: usize) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
        let mut posts: Vec<Post> = self.post_store
            .scan()?
            .filter(|post| post.status == PostStatus::Published)
            .collect();
        
        posts.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        Ok(posts.into_iter().take(limit).collect())
    }
    
    /// Upload media file
    pub fn upload_media(&mut self, media: MediaFile) -> Result<(), Box<dyn std::error::Error>> {
        // Validate uploader exists
        if self.user_store.get(&media.uploaded_by)?.is_none() {
            return Err("Uploader not found".into());
        }
        
        // Validate file type (basic validation)
        let allowed_types = vec![
            "image/jpeg", "image/png", "image/gif", "image/webp",
            "application/pdf", "text/plain"
        ];
        
        if !allowed_types.contains(&media.mime_type.as_str()) {
            return Err("File type not allowed".into());
        }
        
        self.media_store.insert(media.id, media)?;
        Ok(())
    }
    
    /// Subscribe to newsletter
    pub fn subscribe_newsletter(&mut self, subscription: Newsletter) -> Result<(), Box<dyn std::error::Error>> {
        // Check if email already subscribed
        for existing_sub in self.newsletter_store.scan()? {
            if existing_sub.email == subscription.email && existing_sub.is_active {
                return Err("Email already subscribed".into());
            }
        }
        
        self.newsletter_store.insert(subscription.id, subscription)?;
        Ok(())
    }
    
    /// Get blog statistics
    pub fn get_blog_stats(&self) -> Result<BlogStats, Box<dyn std::error::Error>> {
        let total_posts = self.post_store.scan()?.count();
        let published_posts = self.post_store
            .scan()?
            .filter(|p| p.status == PostStatus::Published)
            .count();
        let draft_posts = self.post_store
            .scan()?
            .filter(|p| p.status == PostStatus::Draft)
            .count();
        
        let total_users = self.user_store.scan()?.count();
        let active_users = self.user_store
            .scan()?
            .filter(|u| u.is_active)
            .count();
        
        let total_comments = self.comment_store.scan()?.count();
        let approved_comments = self.comment_store
            .scan()?
            .filter(|c| matches!(c.status, CommentStatus::Approved))
            .count();
        
        let total_categories = self.category_store.scan()?.count();
        let total_tags = self.tag_store.scan()?.count();
        let newsletter_subscribers = self.newsletter_store
            .scan()?
            .filter(|n| n.is_active)
            .count();
        
        Ok(BlogStats {
            total_posts,
            published_posts,
            draft_posts,
            total_users,
            active_users,
            total_comments,
            approved_comments,
            total_categories,
            total_tags,
            newsletter_subscribers,
        })
    }
}

#[derive(Debug)]
pub struct BlogStats {
    pub total_posts: usize,
    pub published_posts: usize,
    pub draft_posts: usize,
    pub total_users: usize,
    pub active_users: usize,
    pub total_comments: usize,
    pub approved_comments: usize,
    pub total_categories: usize,
    pub total_tags: usize,
    pub newsletter_subscribers: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Blog Platform Demo");
    
    let mut blog = BlogPlatform::new()?;
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    
    // Create users with different roles
    println!("\n👥 Creating users...");
    
    let admin = User {
        id: 1,
        username: "admin".to_string(),
        email: "admin@blog.com".to_string(),
        display_name: "Blog Administrator".to_string(),
        role: UserRole::Admin,
        bio: Some("Platform administrator".to_string()),
        avatar_url: None,
        created_at: current_time,
        last_login: Some(current_time),
        is_active: true,
        post_count: 0,
        comment_count: 0,
    };
    blog.create_user(admin)?;
    
    let author = User {
        id: 2,
        username: "jane_author".to_string(),
        email: "jane@blog.com".to_string(),
        display_name: "Jane Smith".to_string(),
        role: UserRole::Author,
        bio: Some("Technical writer and Rust enthusiast".to_string()),
        avatar_url: Some("avatar2.jpg".to_string()),
        created_at: current_time,
        last_login: Some(current_time),
        is_active: true,
        post_count: 0,
        comment_count: 0,
    };
    blog.create_user(author)?;
    
    let subscriber = User {
        id: 3,
        username: "john_reader".to_string(),
        email: "john@example.com".to_string(),
        display_name: "John Doe".to_string(),
        role: UserRole::Subscriber,
        bio: None,
        avatar_url: None,
        created_at: current_time,
        last_login: Some(current_time),
        is_active: true,
        post_count: 0,
        comment_count: 0,
    };
    blog.create_user(subscriber)?;
    
    // Create categories
    println!("📂 Creating categories...");
    
    let programming_category = Category {
        id: 1,
        name: "Programming".to_string(),
        slug: "programming".to_string(),
        description: "Programming tutorials and articles".to_string(),
        parent_id: None,
        post_count: 0,
        created_at: current_time,
        is_active: true,
    };
    blog.create_category(programming_category)?;
    
    let rust_category = Category {
        id: 2,
        name: "Rust".to_string(),
        slug: "rust".to_string(),
        description: "Rust programming language".to_string(),
        parent_id: Some(1), // Child of Programming
        post_count: 0,
        created_at: current_time,
        is_active: true,
    };
    blog.create_category(rust_category)?;
    
    let tutorials_category = Category {
        id: 3,
        name: "Tutorials".to_string(),
        slug: "tutorials".to_string(),
        description: "Step-by-step tutorials".to_string(),
        parent_id: None,
        post_count: 0,
        created_at: current_time,
        is_active: true,
    };
    blog.create_category(tutorials_category)?;
    
    // Create tags
    println!("🏷️ Creating tags...");
    
    let database_tag = Tag {
        id: 1,
        name: "Database".to_string(),
        slug: "database".to_string(),
        description: Some("Database-related content".to_string()),
        usage_count: 0,
        created_at: current_time,
    };
    blog.create_tag(database_tag)?;
    
    let beginner_tag = Tag {
        id: 2,
        name: "Beginner".to_string(),
        slug: "beginner".to_string(),
        description: Some("Beginner-friendly content".to_string()),
        usage_count: 0,
        created_at: current_time,
    };
    blog.create_tag(beginner_tag)?;
    
    let advanced_tag = Tag {
        id: 3,
        name: "Advanced".to_string(),
        slug: "advanced".to_string(),
        description: Some("Advanced topics".to_string()),
        usage_count: 0,
        created_at: current_time,
    };
    blog.create_tag(advanced_tag)?;
    
    // Create blog posts
    println!("📄 Creating blog posts...");
    
    let post1 = Post {
        id: 1,
        title: "Getting Started with NetabaseStore".to_string(),
        slug: "getting-started-netabase".to_string(),
        content: r#"
        NetabaseStore is a powerful database system for Rust applications...
        
        ## Installation
        
        Add to your Cargo.toml:
        ```toml
        [dependencies]
        netabase_store = "0.1.0"
        ```
        
        ## Basic Usage
        
        ```rust
        use netabase_store::prelude::*;
        
        let mut store = UserStore::new_in_memory()?;
        ```
        "#.to_string(),
        excerpt: "Learn how to get started with NetabaseStore in your Rust projects".to_string(),
        author_id: 2,
        category_id: 2, // Rust category
        tag_ids: vec![1, 2], // Database, Beginner
        status: PostStatus::Draft,
        featured_image: Some("netabase-hero.jpg".to_string()),
        meta_description: Some("Getting started guide for NetabaseStore".to_string()),
        view_count: 0,
        like_count: 0,
        comment_count: 0,
        created_at: current_time,
        updated_at: current_time,
        published_at: None,
        scheduled_at: None,
    };
    blog.create_post(post1)?;
    
    let post2 = Post {
        id: 2,
        title: "Advanced Permission Management".to_string(),
        slug: "advanced-permission-management".to_string(),
        content: r#"
        Permission management is a critical aspect of any database system...
        
        ## Hierarchical Permissions
        
        NetabaseStore provides built-in hierarchical permission management:
        
        ```rust
        #[netabase(permissions = "hierarchical")]
        struct Organization {
            id: u64,
            name: String,
        }
        ```
        
        ## Cross-Definition Validation
        
        Ensure data integrity across different stores:
        
        ```rust
        order_store.insert_with_cross_validation(&user_store, order_id, order)?;
        ```
        "#.to_string(),
        excerpt: "Deep dive into NetabaseStore's advanced permission system".to_string(),
        author_id: 2,
        category_id: 2, // Rust category
        tag_ids: vec![1, 3], // Database, Advanced
        status: PostStatus::Draft,
        featured_image: Some("permissions-hero.jpg".to_string()),
        meta_description: Some("Advanced permission management in NetabaseStore".to_string()),
        view_count: 0,
        like_count: 0,
        comment_count: 0,
        created_at: current_time - 3600, // 1 hour ago
        updated_at: current_time - 3600,
        published_at: None,
        scheduled_at: None,
    };
    blog.create_post(post2)?;
    
    // Publish posts (admin/editor action)
    println!("📢 Publishing posts...");
    blog.publish_post(1, 1)?; // Admin publishes post 1
    blog.publish_post(2, 1)?; // Admin publishes post 2
    
    // Add comments
    println!("💬 Adding comments...");
    
    let comment1 = Comment {
        id: 1,
        post_id: 1,
        user_id: 3,
        parent_id: None,
        content: "Great tutorial! This really helped me understand NetabaseStore.".to_string(),
        status: CommentStatus::Pending,
        like_count: 0,
        reply_count: 0,
        created_at: current_time + 1800, // 30 minutes later
        updated_at: current_time + 1800,
        ip_address: "192.168.1.100".to_string(),
        user_agent: "Mozilla/5.0...".to_string(),
    };
    blog.add_comment(comment1)?;
    
    let comment2 = Comment {
        id: 2,
        post_id: 1,
        user_id: 2, // Author replying
        parent_id: Some(1), // Reply to comment 1
        content: "Thanks! I'm glad it was helpful. Let me know if you have any questions.".to_string(),
        status: CommentStatus::Pending,
        like_count: 0,
        reply_count: 0,
        created_at: current_time + 2400, // 40 minutes later
        updated_at: current_time + 2400,
        ip_address: "192.168.1.101".to_string(),
        user_agent: "Mozilla/5.0...".to_string(),
    };
    blog.add_comment(comment2)?;
    
    // Moderate comments (approve them)
    println!("🛡️ Moderating comments...");
    blog.moderate_comment(1, 1, CommentStatus::Approved)?; // Admin approves
    blog.moderate_comment(2, 1, CommentStatus::Approved)?; // Admin approves
    
    // Upload media files
    println!("📷 Uploading media...");
    let media = MediaFile {
        id: 1,
        filename: "netabase-hero.jpg".to_string(),
        original_name: "hero-image.jpg".to_string(),
        mime_type: "image/jpeg".to_string(),
        file_size: 245760, // ~240KB
        width: Some(1200),
        height: Some(630),
        upload_path: "/uploads/2024/01/netabase-hero.jpg".to_string(),
        uploaded_by: 2, // Author uploaded
        alt_text: Some("NetabaseStore hero image".to_string()),
        caption: Some("Getting started with NetabaseStore".to_string()),
        created_at: current_time,
    };
    blog.upload_media(media)?;
    
    // Newsletter subscription
    println!("📧 Creating newsletter subscription...");
    let newsletter = Newsletter {
        id: 1,
        email: "subscriber@example.com".to_string(),
        name: Some("Newsletter Subscriber".to_string()),
        subscribed_at: current_time,
        confirmed_at: Some(current_time + 300), // Confirmed 5 minutes later
        unsubscribed_at: None,
        is_active: true,
        preferences: NewsletterPreferences {
            weekly_digest: true,
            new_posts: true,
            author_updates: false,
            categories: vec![1, 2], // Programming, Rust
        },
    };
    blog.subscribe_newsletter(newsletter)?;
    
    // Demonstrate queries
    println!("\n🔍 Running queries...");
    
    // Get posts by category
    let rust_posts = blog.get_posts_by_category(2)?;
    println!("Rust posts: {}", rust_posts.len());
    
    // Get posts by tag
    let database_posts = blog.get_posts_by_tag(1)?;
    println!("Database posts: {}", database_posts.len());
    
    // Get posts by author
    let author_posts = blog.get_posts_by_author(2)?;
    println!("Author posts: {}", author_posts.len());
    
    // Get comments for a post
    let post_comments = blog.get_post_comments(1)?;
    println!("Post 1 comments: {}", post_comments.len());
    
    // Search posts
    let search_results = blog.search_posts("NetabaseStore")?;
    println!("Search results for 'NetabaseStore': {}", search_results.len());
    
    // Get popular posts
    let popular_posts = blog.get_popular_posts(5)?;
    println!("Popular posts: {}", popular_posts.len());
    
    // Get recent posts
    let recent_posts = blog.get_recent_posts(5)?;
    println!("Recent posts: {}", recent_posts.len());
    
    // Get blog statistics
    println!("\n📊 Blog Statistics:");
    let stats = blog.get_blog_stats()?;
    println!("Total Posts: {}", stats.total_posts);
    println!("Published Posts: {}", stats.published_posts);
    println!("Draft Posts: {}", stats.draft_posts);
    println!("Total Users: {}", stats.total_users);
    println!("Active Users: {}", stats.active_users);
    println!("Total Comments: {}", stats.total_comments);
    println!("Approved Comments: {}", stats.approved_comments);
    println!("Categories: {}", stats.total_categories);
    println!("Tags: {}", stats.total_tags);
    println!("Newsletter Subscribers: {}", stats.newsletter_subscribers);
    
    println!("\n✅ Blog platform demo completed successfully!");
    println!("🎉 All hierarchical permissions and content management working perfectly!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_creation() {
        let mut blog = BlogPlatform::new().unwrap();
        let user = User {
            id: 1,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            display_name: "Test User".to_string(),
            role: UserRole::Subscriber,
            bio: None,
            avatar_url: None,
            created_at: 1640000000,
            last_login: None,
            is_active: true,
            post_count: 0,
            comment_count: 0,
        };
        
        assert!(blog.create_user(user).is_ok());
    }
    
    #[test]
    fn test_category_hierarchy() {
        let mut blog = BlogPlatform::new().unwrap();
        
        let parent_category = Category {
            id: 1,
            name: "Parent".to_string(),
            slug: "parent".to_string(),
            description: "Parent category".to_string(),
            parent_id: None,
            post_count: 0,
            created_at: 1640000000,
            is_active: true,
        };
        blog.create_category(parent_category).unwrap();
        
        let child_category = Category {
            id: 2,
            name: "Child".to_string(),
            slug: "child".to_string(),
            description: "Child category".to_string(),
            parent_id: Some(1),
            post_count: 0,
            created_at: 1640000000,
            is_active: true,
        };
        assert!(blog.create_category(child_category).is_ok());
        
        // Try to create child with non-existent parent
        let orphan_category = Category {
            id: 3,
            name: "Orphan".to_string(),
            slug: "orphan".to_string(),
            description: "Orphan category".to_string(),
            parent_id: Some(999), // Non-existent parent
            post_count: 0,
            created_at: 1640000000,
            is_active: true,
        };
        assert!(blog.create_category(orphan_category).is_err());
    }
    
    #[test]
    fn test_post_creation_validation() {
        let mut blog = BlogPlatform::new().unwrap();
        
        // Setup user and category first
        let author = User {
            id: 1,
            username: "author".to_string(),
            email: "author@example.com".to_string(),
            display_name: "Author".to_string(),
            role: UserRole::Author,
            bio: None,
            avatar_url: None,
            created_at: 1640000000,
            last_login: None,
            is_active: true,
            post_count: 0,
            comment_count: 0,
        };
        blog.create_user(author).unwrap();
        
        let category = Category {
            id: 1,
            name: "Test".to_string(),
            slug: "test".to_string(),
            description: "Test category".to_string(),
            parent_id: None,
            post_count: 0,
            created_at: 1640000000,
            is_active: true,
        };
        blog.create_category(category).unwrap();
        
        let tag = Tag {
            id: 1,
            name: "Test Tag".to_string(),
            slug: "test-tag".to_string(),
            description: None,
            usage_count: 0,
            created_at: 1640000000,
        };
        blog.create_tag(tag).unwrap();
        
        // Valid post
        let post = Post {
            id: 1,
            title: "Test Post".to_string(),
            slug: "test-post".to_string(),
            content: "Test content".to_string(),
            excerpt: "Test excerpt".to_string(),
            author_id: 1,
            category_id: 1,
            tag_ids: vec![1],
            status: PostStatus::Draft,
            featured_image: None,
            meta_description: None,
            view_count: 0,
            like_count: 0,
            comment_count: 0,
            created_at: 1640000000,
            updated_at: 1640000000,
            published_at: None,
            scheduled_at: None,
        };
        assert!(blog.create_post(post).is_ok());
        
        // Invalid post (non-existent author)
        let invalid_post = Post {
            id: 2,
            title: "Invalid Post".to_string(),
            slug: "invalid-post".to_string(),
            content: "Test content".to_string(),
            excerpt: "Test excerpt".to_string(),
            author_id: 999, // Non-existent
            category_id: 1,
            tag_ids: vec![1],
            status: PostStatus::Draft,
            featured_image: None,
            meta_description: None,
            view_count: 0,
            like_count: 0,
            comment_count: 0,
            created_at: 1640000000,
            updated_at: 1640000000,
            published_at: None,
            scheduled_at: None,
        };
        assert!(blog.create_post(invalid_post).is_err());
    }
    
    #[test]
    fn test_comment_threading() {
        let mut blog = BlogPlatform::new().unwrap();
        
        // Setup required entities
        setup_test_entities(&mut blog);
        
        // Parent comment
        let parent_comment = Comment {
            id: 1,
            post_id: 1,
            user_id: 2,
            parent_id: None,
            content: "Parent comment".to_string(),
            status: CommentStatus::Pending,
            like_count: 0,
            reply_count: 0,
            created_at: 1640000000,
            updated_at: 1640000000,
            ip_address: "127.0.0.1".to_string(),
            user_agent: "test".to_string(),
        };
        blog.add_comment(parent_comment).unwrap();
        
        // Child comment
        let child_comment = Comment {
            id: 2,
            post_id: 1,
            user_id: 2,
            parent_id: Some(1),
            content: "Child comment".to_string(),
            status: CommentStatus::Pending,
            like_count: 0,
            reply_count: 0,
            created_at: 1640000100,
            updated_at: 1640000100,
            ip_address: "127.0.0.1".to_string(),
            user_agent: "test".to_string(),
        };
        assert!(blog.add_comment(child_comment).is_ok());
        
        // Verify parent reply count was updated
        let parent = blog.comment_store.get(&1).unwrap().unwrap();
        assert_eq!(parent.reply_count, 1);
    }
    
    #[test]
    fn test_permission_validation() {
        let mut blog = BlogPlatform::new().unwrap();
        setup_test_entities(&mut blog);
        
        // Test publishing permission
        assert!(blog.publish_post(1, 1).is_ok()); // Admin can publish
        assert!(blog.publish_post(1, 2).is_err()); // Subscriber cannot publish
        
        // Test comment moderation permission
        assert!(blog.moderate_comment(1, 1, CommentStatus::Approved).is_ok()); // Admin can moderate
        assert!(blog.moderate_comment(1, 2, CommentStatus::Approved).is_err()); // Subscriber cannot moderate
    }
    
    fn setup_test_entities(blog: &mut BlogPlatform) {
        // Create test admin
        let admin = User {
            id: 1,
            username: "admin".to_string(),
            email: "admin@test.com".to_string(),
            display_name: "Admin".to_string(),
            role: UserRole::Admin,
            bio: None,
            avatar_url: None,
            created_at: 1640000000,
            last_login: None,
            is_active: true,
            post_count: 0,
            comment_count: 0,
        };
        blog.create_user(admin).unwrap();
        
        // Create test subscriber
        let subscriber = User {
            id: 2,
            username: "subscriber".to_string(),
            email: "subscriber@test.com".to_string(),
            display_name: "Subscriber".to_string(),
            role: UserRole::Subscriber,
            bio: None,
            avatar_url: None,
            created_at: 1640000000,
            last_login: None,
            is_active: true,
            post_count: 0,
            comment_count: 0,
        };
        blog.create_user(subscriber).unwrap();
        
        // Create test category
        let category = Category {
            id: 1,
            name: "Test".to_string(),
            slug: "test".to_string(),
            description: "Test category".to_string(),
            parent_id: None,
            post_count: 0,
            created_at: 1640000000,
            is_active: true,
        };
        blog.create_category(category).unwrap();
        
        // Create test tag
        let tag = Tag {
            id: 1,
            name: "Test".to_string(),
            slug: "test".to_string(),
            description: None,
            usage_count: 0,
            created_at: 1640000000,
        };
        blog.create_tag(tag).unwrap();
        
        // Create test post
        let post = Post {
            id: 1,
            title: "Test Post".to_string(),
            slug: "test-post".to_string(),
            content: "Test content".to_string(),
            excerpt: "Test excerpt".to_string(),
            author_id: 1,
            category_id: 1,
            tag_ids: vec![1],
            status: PostStatus::Draft,
            featured_image: None,
            meta_description: None,
            view_count: 0,
            like_count: 0,
            comment_count: 0,
            created_at: 1640000000,
            updated_at: 1640000000,
            published_at: None,
            scheduled_at: None,
        };
        blog.create_post(post).unwrap();
    }
}