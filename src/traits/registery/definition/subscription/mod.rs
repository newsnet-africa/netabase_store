use crate::traits::registery::definition::NetabaseDefinition;

/// Maps a subscription topic discriminant to an array of model discriminants that subscribe to it
#[derive(Debug, Clone, Copy)]
pub struct SubscriptionEntry<TopicDiscriminant, ModelDiscriminant: 'static> {
    pub topic: TopicDiscriminant,
    pub subscribers: &'static [ModelDiscriminant],
}

/// Definition-level subscription registry
/// Maps each subscription topic to the models that subscribe to it
/// This is a const-compatible structure for compile-time subscription routing
pub struct DefinitionSubscriptionRegistry<'a, D: NetabaseDefinition>
where
    D::Discriminant: 'static + std::fmt::Debug,
{
    /// Array of subscription entries mapping topics to model arrays
    pub entries: &'a [SubscriptionEntry<&'static str, D::Discriminant>],
}

impl<'a, D: NetabaseDefinition> DefinitionSubscriptionRegistry<'a, D>
where
    D::Discriminant: 'static + std::fmt::Debug + PartialEq,
{
    /// Create a new subscription registry
    pub const fn new(
        entries: &'a [SubscriptionEntry<&'static str, D::Discriminant>],
    ) -> Self {
        Self { entries }
    }

    /// Get all models that subscribe to a given topic
    pub fn get_subscribers(&self, topic: &str) -> Option<&[D::Discriminant]> {
        self.entries
            .iter()
            .find(|entry| entry.topic == topic)
            .map(|entry| entry.subscribers)
    }

    /// Check if a specific model subscribes to a topic
    pub fn model_subscribes_to(&self, topic: &str, model: D::Discriminant) -> bool {
        self.get_subscribers(topic)
            .map(|subscribers| subscribers.contains(&model))
            .unwrap_or(false)
    }

    /// Get all topics a model subscribes to
    pub fn get_model_topics(&self, model: D::Discriminant) -> Vec<&'static str> {
        self.entries
            .iter()
            .filter(|entry| entry.subscribers.contains(&model))
            .map(|entry| entry.topic)
            .collect()
    }
}
