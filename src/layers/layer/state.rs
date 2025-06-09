use std::{any::Any, collections::HashMap, sync::Arc};

#[allow(dead_code)]
pub struct LayerDataProps(HashMap<String, Arc<dyn Any + Send + Sync>>);

impl LayerDataProps {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    #[allow(dead_code)]
    pub fn insert<V: Clone + Send + Sync + 'static>(&mut self, key: impl AsRef<str>, value: V) {
        let key = key.as_ref();
        self.0.insert(key.to_string(), Arc::new(value));
    }
    #[allow(dead_code)]
    pub fn get<V: Clone + 'static>(&self, key: impl AsRef<str>) -> Option<V> {
        let key = key.as_ref();
        self.0.get(key).and_then(|v| v.downcast_ref::<V>()).cloned()
    }
}

impl Default for LayerDataProps {
    fn default() -> Self {
        Self::new()
    }
}
