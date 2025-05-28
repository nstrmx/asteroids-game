#![allow(dead_code)]

use std::collections::HashMap;

use crate::EntityId;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum EventType {
	EntityDestroyed,
	NumberOfAsteroidsIncreased,
	ResetAsteroid,
	ScoreIncreased,
	GameOver,
}

pub enum Event {
	EntityDestroyed(EntityId),
	NumberOfAsteroidsIncreased,
	ResetAsteroid(EntityId),
	ScoreIncreased,
	GameOver,
}

type EventCallback = Box<dyn Fn(&Event, &mut crate::Game)>;

pub struct EventBus {
	listeners: HashMap<EventType, Vec<EventCallback>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
        }    
    }
    
    pub fn subscribe(&mut self, event_type: EventType, callback: EventCallback) {
        if let Some(callbacks) = self.listeners.get_mut(&event_type){
        	callbacks.push(callback);    
        } else {
            self.listeners.insert(event_type, vec![callback]);
        }
    }    

    pub fn publish(&mut self, event_type: EventType, event: Event, ctx: &mut crate::Game) {
    	for callback in self.listeners.get_mut(&event_type).unwrap() {
    		callback(&event, ctx);
    	}
    }
}
