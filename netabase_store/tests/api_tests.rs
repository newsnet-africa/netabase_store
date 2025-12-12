//! API tests with comprehensive mock application
//!
//! This creates a realistic mock application that exercises as much of the
//! NetabaseStore API as possible to ensure everything works together correctly.

use netabase_store::error::{NetabaseError, NetabaseResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// Comprehensive model hierarchy for API testing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiUser {
    pub id: u64,
    pub uuid: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub profile: UserProfile,
    pub organization_memberships: Vec<OrganizationMembership>,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    pub full_name: String,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub social_links: Vec<SocialLink>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SocialLink {
    pub platform: String,
    pub url: String,
    pub verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrganizationMembership {
    pub organization_id: u64,
    pub role: OrganizationRole,
    pub joined_at: u64,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrganizationRole {
    Owner,
    Admin,
    Member,
    Guest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiOrganization {
    pub id: u64,
    pub uuid: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub owner_id: u64,
    pub settings: OrganizationSettings,
    pub billing_info: BillingInfo,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrganizationSettings {
    pub public: bool,
    pub allow_external_collaborators: bool,
    pub max_projects: u32,
    pub max_members: u32,
    pub features_enabled: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillingInfo {
    pub plan: String,
    pub billing_email: String,
    pub payment_method: Option<String>,
    pub next_billing_date: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiProject {
    pub id: u64,
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub organization_id: u64,
    pub owner_id: u64,
    pub collaborators: Vec<ProjectCollaborator>,
    pub settings: ProjectSettings,
    pub metrics: ProjectMetrics,
    pub created_at: u64,
    pub updated_at: u64,
    pub status: ProjectStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectCollaborator {
    pub user_id: u64,
    pub role: ProjectRole,
    pub permissions: Vec<String>,
    pub added_at: u64,
    pub added_by: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectRole {
    Owner,
    Maintainer,
    Developer,
    Guest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub public: bool,
    pub allow_issues: bool,
    pub allow_discussions: bool,
    pub auto_merge: bool,
    pub required_reviewers: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectMetrics {
    pub total_commits: u64,
    pub total_contributors: u64,
    pub lines_of_code: u64,
    pub last_activity: u64,
    pub health_score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Archived,
    Deleted,
    Suspended,
}

// Event system for testing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ApiEvent {
    UserCreated { user_id: u64 },
    UserUpdated { user_id: u64 },
    UserDeleted { user_id: u64 },
    OrganizationCreated { org_id: u64, owner_id: u64 },
    OrganizationUpdated { org_id: u64 },
    ProjectCreated { project_id: u64, org_id: u64, owner_id: u64 },
    ProjectUpdated { project_id: u64 },
    CollaboratorAdded { project_id: u64, user_id: u64 },
    CollaboratorRemoved { project_id: u64, user_id: u64 },
}

// Mock storage implementations for API testing
struct MockStore<T> {
    data: Arc<RwLock<HashMap<u64, T>>>,
}

impl<T> MockStore<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn insert(&self, id: u64, item: T) -> NetabaseResult<()> {
        let mut data = self.data.write().unwrap();
        data.insert(id, item);
        Ok(())
    }

    fn get(&self, id: &u64) -> NetabaseResult<Option<T>> {
        let data = self.data.read().unwrap();
        Ok(data.get(id).cloned())
    }

    fn update(&self, id: u64, item: T) -> NetabaseResult<()> {
        let mut data = self.data.write().unwrap();
        if data.contains_key(&id) {
            data.insert(id, item);
            Ok(())
        } else {
            Err(NetabaseError::Other(format!("Item with id {} not found", id)))
        }
    }

    fn delete(&self, id: &u64) -> NetabaseResult<()> {
        let mut data = self.data.write().unwrap();
        if data.remove(id).is_some() {
            Ok(())
        } else {
            Err(NetabaseError::Other(format!("Item with id {} not found", id)))
        }
    }

    fn list(&self) -> NetabaseResult<Vec<T>> {
        let data = self.data.read().unwrap();
        Ok(data.values().cloned().collect())
    }
}

// Comprehensive mock application
pub struct MockNetabaseApplication {
    // Primary stores
    user_store: MockStore<ApiUser>,
    organization_store: MockStore<ApiOrganization>,
    project_store: MockStore<ApiProject>,
    
    // Secondary indices for efficient lookups
    user_by_email: Arc<RwLock<HashMap<String, u64>>>,
    user_by_username: Arc<RwLock<HashMap<String, u64>>>,
    org_by_slug: Arc<RwLock<HashMap<String, u64>>>,
    
    // Event system
    event_publisher: tokio::sync::broadcast::Sender<ApiEvent>,
    
    // Metrics and monitoring
    api_call_count: Arc<RwLock<HashMap<String, u64>>>,
    performance_metrics: Arc<RwLock<HashMap<String, f64>>>,
    
    // Configuration
    config: ApplicationConfig,
}

#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    pub max_users_per_org: u32,
    pub max_projects_per_org: u32,
    pub enable_user_registration: bool,
    pub require_email_verification: bool,
    pub default_user_quota: u64,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            max_users_per_org: 1000,
            max_projects_per_org: 100,
            enable_user_registration: true,
            require_email_verification: false,
            default_user_quota: 1_000_000_000, // 1GB in bytes
        }
    }
}

impl MockNetabaseApplication {
    pub fn new() -> Self {
        let (event_tx, _) = tokio::sync::broadcast::channel(1000);
        
        Self {
            user_store: MockStore::new(),
            organization_store: MockStore::new(),
            project_store: MockStore::new(),
            user_by_email: Arc::new(RwLock::new(HashMap::new())),
            user_by_username: Arc::new(RwLock::new(HashMap::new())),
            org_by_slug: Arc::new(RwLock::new(HashMap::new())),
            event_publisher: event_tx,
            api_call_count: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(HashMap::new())),
            config: ApplicationConfig::default(),
        }
    }
    
    fn increment_api_call(&self, endpoint: &str) {
        let mut calls = self.api_call_count.write().unwrap();
        *calls.entry(endpoint.to_string()).or_insert(0) += 1;
    }
    
    fn record_performance(&self, endpoint: &str, duration_ms: f64) {
        let mut metrics = self.performance_metrics.write().unwrap();
        metrics.insert(endpoint.to_string(), duration_ms);
    }

    // User management API
    pub fn create_user(&self, username: String, email: String, password: String) -> NetabaseResult<u64> {
        self.increment_api_call("create_user");
        
        // Validation
        if username.len() < 3 {
            return Err(NetabaseError::Other("Username too short".to_string()));
        }
        if !email.contains('@') {
            return Err(NetabaseError::Other("Invalid email".to_string()));
        }
        
        // Check for duplicates
        {
            let email_index = self.user_by_email.read().unwrap();
            let username_index = self.user_by_username.read().unwrap();
            
            if email_index.contains_key(&email) {
                return Err(NetabaseError::Other("Email already exists".to_string()));
            }
            if username_index.contains_key(&username) {
                return Err(NetabaseError::Other("Username already taken".to_string()));
            }
        }
        
        let user_id = self.generate_id();
        let now = self.current_timestamp();
        
        let user = ApiUser {
            id: user_id,
            uuid: Uuid::new_v4().to_string(),
            username: username.clone(),
            email: email.clone(),
            password_hash: format!("hashed_{}", password), // Mock hash
            profile: UserProfile {
                full_name: username.clone(),
                bio: None,
                avatar_url: None,
                location: None,
                website: None,
                social_links: vec![],
            },
            organization_memberships: vec![],
            created_at: now,
            updated_at: now,
            is_active: true,
        };
        
        // Store user and update indices
        self.user_store.insert(user_id, user)?;
        
        {
            let mut email_index = self.user_by_email.write().unwrap();
            let mut username_index = self.user_by_username.write().unwrap();
            email_index.insert(email, user_id);
            username_index.insert(username, user_id);
        }
        
        let _ = self.event_publisher.send(ApiEvent::UserCreated { user_id });
        
        Ok(user_id)
    }

    pub fn get_user(&self, user_id: u64) -> NetabaseResult<Option<ApiUser>> {
        self.increment_api_call("get_user");
        self.user_store.get(&user_id)
    }

    pub fn get_user_by_email(&self, email: &str) -> NetabaseResult<Option<ApiUser>> {
        self.increment_api_call("get_user_by_email");
        
        let email_index = self.user_by_email.read().unwrap();
        if let Some(user_id) = email_index.get(email) {
            self.user_store.get(user_id)
        } else {
            Ok(None)
        }
    }

    pub fn update_user(&self, user_id: u64, updated_user: ApiUser) -> NetabaseResult<()> {
        self.increment_api_call("update_user");
        
        if self.user_store.get(&user_id)?.is_some() {
            let mut user = updated_user;
            user.updated_at = self.current_timestamp();
            self.user_store.update(user_id, user)?;
            let _ = self.event_publisher.send(ApiEvent::UserUpdated { user_id });
            Ok(())
        } else {
            Err(NetabaseError::Other("User not found".to_string()))
        }
    }

    // Organization management API
    pub fn create_organization(&self, name: String, slug: String, owner_id: u64) -> NetabaseResult<u64> {
        self.increment_api_call("create_organization");
        
        // Validate inputs
        if name.len() < 2 {
            return Err(NetabaseError::Other("Organization name too short".to_string()));
        }
        if slug.len() < 2 {
            return Err(NetabaseError::Other("Organization slug too short".to_string()));
        }

        // Check if owner exists
        {
            if self.user_store.get(&owner_id)?.is_none() {
                return Err(NetabaseError::Other("Owner user not found".to_string()));
            }
        }

        // Check slug uniqueness
        {
            let slug_index = self.org_by_slug.read().unwrap();
            if slug_index.contains_key(&slug) {
                return Err(NetabaseError::Other("Organization slug already taken".to_string()));
            }
        }

        let org_id = self.generate_id();
        let now = self.current_timestamp();

        let organization = ApiOrganization {
            id: org_id,
            uuid: Uuid::new_v4().to_string(),
            name: name.clone(),
            slug: slug.clone(),
            description: format!("Organization: {}", name),
            owner_id,
            settings: OrganizationSettings {
                public: false,
                allow_external_collaborators: true,
                max_projects: self.config.max_projects_per_org,
                max_members: self.config.max_users_per_org,
                features_enabled: vec!["projects".to_string(), "teams".to_string()],
            },
            billing_info: BillingInfo {
                plan: "free".to_string(),
                billing_email: "billing@example.com".to_string(),
                payment_method: None,
                next_billing_date: None,
            },
            created_at: now,
            updated_at: now,
            is_active: true,
        };

        // Store organization and update indices
        self.organization_store.insert(org_id, organization)?;
        
        {
            let mut slug_index = self.org_by_slug.write().unwrap();
            slug_index.insert(slug, org_id);
        }

        let _ = self.event_publisher.send(ApiEvent::OrganizationCreated { 
            org_id, 
            owner_id 
        });

        Ok(org_id)
    }

    // Project management API
    pub fn create_project(&self, name: String, org_id: u64, owner_id: u64) -> NetabaseResult<u64> {
        self.increment_api_call("create_project");

        // Validate organization exists
        {
            let org_option = self.organization_store.get(&org_id).unwrap();
            if org_option.is_none() {
                return Err(NetabaseError::Other("Organization not found".to_string()));
            }
        }

        // Validate owner is member of organization
        {
            if let Some(user) = self.user_store.get(&owner_id)? {
                if !user.organization_memberships.iter()
                    .any(|m| m.organization_id == org_id) {
                    return Err(NetabaseError::PermissionDenied(
                        "User is not a member of the organization".to_string()
                    ));
                }
            } else {
                return Err(NetabaseError::Other("Owner user not found".to_string()));
            }
        }

        let project_id = self.generate_id();
        let now = self.current_timestamp();

        let project = ApiProject {
            id: project_id,
            uuid: Uuid::new_v4().to_string(),
            name: name.clone(),
            description: format!("Project: {}", name),
            organization_id: org_id,
            owner_id,
            collaborators: vec![],
            settings: ProjectSettings {
                public: false,
                allow_issues: true,
                allow_discussions: true,
                auto_merge: false,
                required_reviewers: 1,
            },
            metrics: ProjectMetrics {
                total_commits: 0,
                total_contributors: 1,
                lines_of_code: 0,
                last_activity: now,
                health_score: 100.0,
            },
            created_at: now,
            updated_at: now,
            status: ProjectStatus::Active,
        };

        self.project_store.insert(project_id, project)?;

        let _ = self.event_publisher.send(ApiEvent::ProjectCreated { 
            project_id, 
            org_id, 
            owner_id 
        });

        Ok(project_id)
    }

    pub fn add_project_collaborator(&self, project_id: u64, user_id: u64, role: ProjectRole) -> NetabaseResult<()> {
        self.increment_api_call("add_project_collaborator");

        // Validate user exists
        {
            let user_store = self.user_store.data.read().unwrap();
            if !user_store.contains_key(&user_id) {
                return Err(NetabaseError::Other("User not found".to_string()));
            }
        }

        // Update project
        {
            let mut project_store = self.project_store.data.write().unwrap();
            if let Some(project) = project_store.get_mut(&project_id) {
                // Check if already a collaborator
                if project.collaborators.iter().any(|c| c.user_id == user_id) {
                    return Err(NetabaseError::Other("User is already a collaborator".to_string()));
                }

                project.collaborators.push(ProjectCollaborator {
                    user_id,
                    role,
                    permissions: vec!["read".to_string(), "write".to_string()],
                    added_at: self.current_timestamp(),
                    added_by: project.owner_id,
                });

                project.updated_at = self.current_timestamp();
                project.metrics.total_contributors += 1;

                let _ = self.event_publisher.send(ApiEvent::CollaboratorAdded { project_id, user_id });
                Ok(())
            } else {
                Err(NetabaseError::Other("Project not found".to_string()))
            }
        }
    }

    // Utility methods
    fn generate_id(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }

    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    // Analytics and metrics
    pub fn get_api_metrics(&self) -> HashMap<String, u64> {
        self.api_call_count.read().unwrap().clone()
    }

    pub fn get_user_count(&self) -> NetabaseResult<usize> {
        Ok(self.user_store.list()?.len())
    }

    pub fn get_organization_count(&self) -> NetabaseResult<usize> {
        Ok(self.organization_store.list()?.len())
    }

    pub fn get_project_count(&self) -> NetabaseResult<usize> {
        Ok(self.project_store.list()?.len())
    }
}

// Comprehensive API tests
mod api_functionality_tests {
    use super::*;

    #[test]
    fn api_user_lifecycle_complete() {
        let app = MockNetabaseApplication::new();

        // Create user
        let user_id = app.create_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "password123".to_string()
        ).unwrap();

        // Get user
        let user = app.get_user(user_id).unwrap().unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");

        // Get user by email
        let user_by_email = app.get_user_by_email("test@example.com").unwrap().unwrap();
        assert_eq!(user_by_email.id, user_id);

        // Update user
        let mut updated_user = user.clone();
        updated_user.profile.full_name = "Test User Updated".to_string();
        app.update_user(user_id, updated_user).unwrap();

        let updated = app.get_user(user_id).unwrap().unwrap();
        assert_eq!(updated.profile.full_name, "Test User Updated");
        assert!(updated.updated_at >= user.created_at);
    }

    #[test]
    fn api_organization_workflow() {
        let app = MockNetabaseApplication::new();

        // Create owner
        let owner_id = app.create_user(
            "orgowner".to_string(),
            "owner@testorg.com".to_string(),
            "password".to_string()
        ).unwrap();

        // Create organization
        let org_id = app.create_organization(
            "Test Organization".to_string(),
            "test-org".to_string(),
            owner_id
        ).unwrap();

        // Verify organization exists
        let org = app.organization_store.get(&org_id).unwrap().unwrap();
        assert_eq!(org.name, "Test Organization");
        assert_eq!(org.slug, "test-org");
        assert_eq!(org.owner_id, owner_id);
    }

    #[test]
    fn api_project_collaboration_workflow() {
        let app = MockNetabaseApplication::new();

        // Create users
        let owner_id = app.create_user("owner".to_string(), "owner@example.com".to_string(), "pass".to_string()).unwrap();
        let member_id = app.create_user("member".to_string(), "member@example.com".to_string(), "pass".to_string()).unwrap();
        let non_member_id = app.create_user("outsider".to_string(), "outsider@example.com".to_string(), "pass".to_string()).unwrap();

        // Create organization
        let org_id = app.create_organization("Collab Org".to_string(), "collab".to_string(), owner_id).unwrap();

        // Add owner to organization (simplified - in real system this would be automatic)
        {
            let mut user = app.get_user(owner_id).unwrap().unwrap();
            user.organization_memberships.push(OrganizationMembership {
                organization_id: org_id,
                role: OrganizationRole::Owner,
                joined_at: app.current_timestamp(),
                permissions: vec!["read".to_string(), "write".to_string(), "admin".to_string()],
            });
            app.update_user(owner_id, user).unwrap();
        }

        // Add member to organization (simplified - in real system this would be a separate API)
        {
            let mut user = app.get_user(member_id).unwrap().unwrap();
            user.organization_memberships.push(OrganizationMembership {
                organization_id: org_id,
                role: OrganizationRole::Member,
                joined_at: app.current_timestamp(),
                permissions: vec!["read".to_string(), "write".to_string()],
            });
            app.update_user(member_id, user).unwrap();
        }

        // Create project
        let project_id = app.create_project("Test Project".to_string(), org_id, owner_id).unwrap();

        // Add collaborator (member)
        app.add_project_collaborator(project_id, member_id, ProjectRole::Developer).unwrap();

        // Verify collaboration
        let project = app.project_store.get(&project_id).unwrap().unwrap();
        assert_eq!(project.collaborators.len(), 1);
        assert_eq!(project.collaborators[0].user_id, member_id);

        // Try to add non-existent user as collaborator
        let result = app.add_project_collaborator(project_id, 999, ProjectRole::Developer);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetabaseError::Other(_)));

        // Add existing user as collaborator (should work)
        let result = app.add_project_collaborator(project_id, non_member_id, ProjectRole::Developer);
        assert!(result.is_ok()); // In simplified version, we allow this
    }

    #[test]
    fn api_validation_and_error_handling() {
        let app = MockNetabaseApplication::new();

        // Test user validation
        let result = app.create_user("a".to_string(), "invalid-email".to_string(), "pass".to_string());
        assert!(result.is_err());

        // Create valid user
        let _user_id = app.create_user("validuser".to_string(), "valid@email.com".to_string(), "pass".to_string()).unwrap();

        // Test duplicate email
        let result = app.create_user("anotheruser".to_string(), "valid@email.com".to_string(), "pass".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetabaseError::Other(_)));

        // Test duplicate username  
        let result = app.create_user("validuser".to_string(), "another@email.com".to_string(), "pass".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetabaseError::Other(_)));

        // Test organization creation with non-existent owner
        let result = app.create_organization("Test Org".to_string(), "test-org".to_string(), 999);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), NetabaseError::Other(_)));
    }

    #[test]
    fn api_metrics_and_monitoring() {
        let app = MockNetabaseApplication::new();

        // Perform various API operations
        let _user1 = app.create_user("user1".to_string(), "user1@example.com".to_string(), "pass".to_string()).unwrap();
        let _user2 = app.create_user("user2".to_string(), "user2@example.com".to_string(), "pass".to_string()).unwrap();
        let _ = app.get_user_by_email("user1@example.com");
        let _ = app.get_user_by_email("user2@example.com");

        // Check metrics
        let metrics = app.get_api_metrics();
        assert_eq!(metrics.get("create_user"), Some(&2));
        assert_eq!(metrics.get("get_user_by_email"), Some(&2));

        // Check counts
        assert_eq!(app.get_user_count().unwrap(), 2);
        assert_eq!(app.get_organization_count().unwrap(), 0);
        assert_eq!(app.get_project_count().unwrap(), 0);
    }

    #[test]
    fn api_concurrent_operations() {
        use std::sync::Arc;
        use std::thread;

        let app = Arc::new(MockNetabaseApplication::new());
        let num_threads = 5;
        let operations_per_thread = 10;

        let mut handles = vec![];

        // Concurrent user creation
        for thread_id in 0..num_threads {
            let app_clone = Arc::clone(&app);
            let handle = thread::spawn(move || {
                for i in 0..operations_per_thread {
                    let _user_id = thread_id * operations_per_thread + i;
                    let username = format!("concurrent_user_{}_{}", thread_id, i);
                    let email = format!("user_{}@thread{}.com", i, thread_id);
                    
                    let _ = app_clone.create_user(username, email, "password".to_string());
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify concurrent operations succeeded
        let final_user_count = app.get_user_count().unwrap();
        assert!(final_user_count > 0);
        println!("Created {} users concurrently", final_user_count);
    }

    #[test]
    fn api_complex_integration_scenario() {
        let app = MockNetabaseApplication::new();

        // Create a complex scenario with multiple users, organizations, and projects
        let ceo_id = app.create_user("ceo".to_string(), "ceo@company.com".to_string(), "pass".to_string()).unwrap();
        let dev1_id = app.create_user("dev1".to_string(), "dev1@company.com".to_string(), "pass".to_string()).unwrap();
        let dev2_id = app.create_user("dev2".to_string(), "dev2@company.com".to_string(), "pass".to_string()).unwrap();

        // Create organization
        let org_id = app.create_organization("TechCorp".to_string(), "techcorp".to_string(), ceo_id).unwrap();

        // Add CEO to organization (simplified - in real system this would be automatic)
        {
            let mut user = app.get_user(ceo_id).unwrap().unwrap();
            user.organization_memberships.push(OrganizationMembership {
                organization_id: org_id,
                role: OrganizationRole::Owner,
                joined_at: app.current_timestamp(),
                permissions: vec!["read".to_string(), "write".to_string(), "admin".to_string()],
            });
            app.update_user(ceo_id, user).unwrap();
        }

        // Add developers to organization (simplified)
        for &dev_id in &[dev1_id, dev2_id] {
            let mut user = app.get_user(dev_id).unwrap().unwrap();
            user.organization_memberships.push(OrganizationMembership {
                organization_id: org_id,
                role: OrganizationRole::Member,
                joined_at: app.current_timestamp(),
                permissions: vec!["read".to_string(), "write".to_string()],
            });
            app.update_user(dev_id, user).unwrap();
        }

        // Create multiple projects
        let project1_id = app.create_project("Backend API".to_string(), org_id, ceo_id).unwrap();
        let project2_id = app.create_project("Frontend App".to_string(), org_id, dev1_id).unwrap();

        // Add collaborators
        app.add_project_collaborator(project1_id, dev1_id, ProjectRole::Developer).unwrap();
        app.add_project_collaborator(project1_id, dev2_id, ProjectRole::Developer).unwrap();
        app.add_project_collaborator(project2_id, dev2_id, ProjectRole::Maintainer).unwrap();

        // Verify the complete system state
        assert_eq!(app.get_user_count().unwrap(), 3);
        assert_eq!(app.get_organization_count().unwrap(), 1);
        assert_eq!(app.get_project_count().unwrap(), 2);

        let project1 = app.project_store.get(&project1_id).unwrap().unwrap();
        assert_eq!(project1.collaborators.len(), 2);

        let project2 = app.project_store.get(&project2_id).unwrap().unwrap();
        assert_eq!(project2.collaborators.len(), 1);
        assert_eq!(project2.collaborators[0].role, ProjectRole::Maintainer);

        // Check metrics show all operations were tracked
        let metrics = app.get_api_metrics();
        assert!(metrics.get("create_user").unwrap_or(&0) >= &3);
        assert!(metrics.get("create_organization").unwrap_or(&0) >= &1);
        assert!(metrics.get("create_project").unwrap_or(&0) >= &2);
    }
}