#![allow(dead_code)]

use std::collections::HashMap;
use crate::Context;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
enum EventType {
	AsteroidDestroyed,
	PlayerHit,
	GameOver,
}

enum Event {
	AsteroidDestroyed{asteroid_id: usize, laser_id: usize},
	PlayerHit{asteroid_id: usize},
	GameOver,
}

type EventCallback = Box<dyn Fn(&Event, &mut Context)>;

pub struct EventBus {
	listeners: HashMap<EventType, Vec<EventCallback>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
        }    
    }
    
    fn subscribe(&mut self, event_type: EventType, callback: EventCallback) {
        if let Some(callbacks) = self.listeners.get_mut(&event_type){
        	callbacks.push(callback);    
        } else {
            self.listeners.insert(event_type, vec![callback]);
        }
    }    

    fn publish(&mut self, event_type: EventType, event: Event, ctx: &mut Context) {
    	for callback in self.listeners.get_mut(&event_type).unwrap() {
    		callback(&event, ctx);
    	}
    }
}
