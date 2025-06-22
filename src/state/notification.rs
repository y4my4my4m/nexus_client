use common::Notification;

/// State management for notifications
pub struct NotificationState {
    pub notifications: Vec<Notification>,
    pub notification_history_complete: bool,
    pub current_notification: Option<(String, Option<u64>, bool)>, // message, close_tick, minimal
}

impl Default for NotificationState {
    fn default() -> Self {
        Self {
            notifications: Vec::new(),
            notification_history_complete: false,
            current_notification: None,
        }
    }
}

impl NotificationState {
    pub fn set_notification(&mut self, message: impl Into<String>, ms: Option<u64>, minimal: bool, tick_count: u64) {
        let close_tick = ms.map(|duration| tick_count + duration);
        self.current_notification = Some((message.into(), close_tick, minimal));
    }
    
    pub fn clear_notification(&mut self) {
        self.current_notification = None;
    }
    
    // pub fn update_notification(&mut self, notification_id: Uuid, read: bool) {
    //     if let Some(n) = self.notifications.iter_mut().find(|n| n.id == notification_id) {
    //         n.read = read;
    //     }
    // }

    pub fn should_close_notification(&self, tick_count: u64) -> bool {
        if let Some((_, Some(close_tick), _)) = &self.current_notification {
            tick_count >= *close_tick
        } else {
            false
        }
    }
}