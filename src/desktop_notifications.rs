use notify_rust::{Notification, Timeout};
use crate::global_prefs::global_prefs;
use tracing::{debug, error};
use std::fs;
use base64::Engine;

/// Desktop notification service for system-level notifications
pub struct DesktopNotificationService;

impl DesktopNotificationService {
    /// Show a desktop notification with the given title and message
    pub fn show_notification(title: &str, message: &str, urgency: NotificationUrgency) {
        Self::show_notification_with_icon(title, message, urgency, None);
    }

    /// Show a desktop notification with an optional custom icon
    pub fn show_notification_with_icon(title: &str, message: &str, urgency: NotificationUrgency, icon_path: Option<String>) {
        // Check if notifications are enabled in preferences
        let prefs = global_prefs();
        if !prefs.desktop_notifications_enabled {
            debug!("Desktop notifications disabled in preferences");
            return;
        }

        // Show notification in a separate task to avoid blocking
        let title = title.to_string();
        let message = message.to_string();
        
        tokio::spawn(async move {
            if let Err(e) = Self::send_notification(&title, &message, urgency, icon_path).await {
                error!("Failed to send desktop notification: {}", e);
            }
        });
    }

    /// Show a direct message notification with sender's profile picture
    pub fn show_dm_notification(from_username: &str, message_preview: &str, sender_profile_pic: Option<&String>) {
        let message = if message_preview.len() > 100 {
            format!("{}...", &message_preview[..97])
        } else {
            message_preview.to_string()
        };
        
        // Convert Option<&String> to Option<&str> for the helper function
        let profile_pic_str = sender_profile_pic.map(|s| s.as_str());
        let icon_path = Self::prepare_profile_picture_icon(profile_pic_str, from_username);
        
        Self::show_notification_with_icon(&from_username, &message, NotificationUrgency::Normal, icon_path);
    }

    /// Show a mention notification with sender's profile picture
    pub fn show_mention_notification(from_username: &str, content: &str, sender_profile_pic: Option<&str>) {
        let title = format!("Mentioned by {}", from_username);
        let message = if content.len() > 100 {
            format!("{}...", &content[..97])
        } else {
            content.to_string()
        };
        
        let icon_path = Self::prepare_profile_picture_icon(sender_profile_pic, from_username);
        
        Self::show_notification_with_icon(&title, &message, NotificationUrgency::Normal, icon_path);
    }

    /// Show a server invite notification
    pub fn show_server_invite_notification(from_username: &str, server_name: &str) {
        let title = "Server Invite Received";
        let message = format!("{} invited you to join '{}'", from_username, server_name);
        
        Self::show_notification(&title, &message, NotificationUrgency::Normal);
    }

    /// Show a general info notification
    pub fn show_info_notification(message: &str) {
        Self::show_notification("Nexus", message, NotificationUrgency::Low);
    }

    /// Show an error notification
    pub fn show_error_notification(message: &str) {
        let title = "Nexus - Error";
        Self::show_notification(title, message, NotificationUrgency::Critical);
    }

    /// Prepare a profile picture as a temporary icon file for notifications
    fn prepare_profile_picture_icon(profile_pic_base64: Option<&str>, username: &str) -> Option<String> {
        let profile_pic_data = match profile_pic_base64 {
            Some(data) => {
                data
            }
            None => {
                return None;
            }
        };
        
        // Extract base64 data from data URL if present
        let base64_data = if let Some(comma_pos) = profile_pic_data.find(',') {
            &profile_pic_data[comma_pos + 1..]
        } else {
            profile_pic_data
        };
        
        // Decode base64 to bytes
        let image_bytes = match base64::engine::general_purpose::STANDARD.decode(base64_data) {
            Ok(bytes) => {
                bytes
            }
            Err(_e) => {
                return None;
            }
        };
        
        // Process the image to create a circular icon
        let processed_icon = match Self::create_circular_notification_icon(&image_bytes, 48) {
            Ok(icon_bytes) => {
                icon_bytes
            }
            Err(_e) => {
                return None;
            }
        };
        
        // Create icon in a more accessible location and with better naming
        let icon_dir = std::env::temp_dir();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let safe_username: String = username.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .take(20)
            .collect();
        let icon_filename = format!("cyberpunk_bbs_{}_{}.png", safe_username, timestamp);
        let icon_path = icon_dir.join(icon_filename);
        
        match fs::write(&icon_path, &processed_icon) {
            Ok(()) => {
                // Set file permissions to be readable by all (for notification daemon)
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Err(_e) = fs::set_permissions(&icon_path, fs::Permissions::from_mode(0o644)) {
                        // handle error
                    }
                }
                
                // Verify the file was created correctly
                if let Ok(_metadata) = fs::metadata(&icon_path) {
                    Some(icon_path.to_string_lossy().to_string())
                } else {
                    None
                }
            }
            Err(_e) => {
                None
            }
        }
    }
    
    /// Create a circular icon suitable for desktop notifications
    fn create_circular_notification_icon(image_bytes: &[u8], size: u32) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // Load the image
        let img = image::load_from_memory(image_bytes)?;
        
        // Resize to target size maintaining aspect ratio, then crop to square
        let resized = img.resize_to_fill(size, size, image::imageops::FilterType::Lanczos3);
        let mut rgba_img = resized.to_rgba8();
        
        // Apply circular mask
        let center_x = size as f32 / 2.0;
        let center_y = size as f32 / 2.0;
        let radius = size as f32 / 2.0 - 1.0;
        
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                let pixel = rgba_img.get_pixel_mut(x, y);
                if distance > radius {
                    // Outside circle - make transparent
                    pixel[3] = 0;
                } else if distance > radius - 2.0 {
                    // Edge anti-aliasing
                    let fade = (radius - distance) / 2.0;
                    pixel[3] = (pixel[3] as f32 * fade) as u8;
                }
            }
        }
        
        // Add subtle border
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance >= radius - 2.0 && distance <= radius {
                    let pixel = rgba_img.get_pixel_mut(x, y);
                    pixel[0] = ((pixel[0] as u16 + 255) / 2) as u8;
                    pixel[1] = ((pixel[1] as u16 + 255) / 2) as u8;
                    pixel[2] = ((pixel[2] as u16 + 255) / 2) as u8;
                    pixel[3] = 255;
                }
            }
        }
        
        // Convert back to PNG bytes
        let mut buffer = Vec::new();
        rgba_img.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)?;
        Ok(buffer)
    }

    async fn send_notification(title: &str, message: &str, urgency: NotificationUrgency, icon_path: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut notification = Notification::new();
        
        notification
            .summary(title)
            .body(message)
            .appname("Nexus")
            .timeout(match urgency {
                NotificationUrgency::Low => Timeout::Milliseconds(3000),
                NotificationUrgency::Normal => Timeout::Milliseconds(5000),
                NotificationUrgency::Critical => Timeout::Milliseconds(8000),
            });

        // Set custom icon if provided, otherwise use default
        if let Some(ref icon) = icon_path {
            // Check if the icon file actually exists and is readable
            if std::path::Path::new(icon).exists() {
                notification.icon(icon);
            } else {
                notification.icon("dialog-information"); // Fallback to standard icon
            }
        } else {
            notification.icon("dialog-information"); // Standard fallback icon
        }

        // Set urgency level for systems that support it
        #[cfg(target_os = "linux")]
        {
            use notify_rust::Urgency;
            let urgency_level = match urgency {
                NotificationUrgency::Low => Urgency::Low,
                NotificationUrgency::Normal => Urgency::Normal,
                NotificationUrgency::Critical => Urgency::Critical,
            };
            notification.urgency(urgency_level);
        }

        // Show the notification
        let handle = notification.show()?;
        
        debug!("Desktop notification sent: {} - {} (icon: {:?})", title, message, icon_path.as_ref().map(|p| p.as_str()).unwrap_or("default"));
        
        // Clean up temporary icon file after a longer delay to ensure it's been used
        if let Some(icon) = icon_path {
            tokio::spawn(async move {
                // Wait longer for KDE to process the icon
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                if let Err(e) = fs::remove_file(&icon) {
                    debug!("Failed to clean up notification icon file {}: {}", icon, e);
                } else {
                    debug!("Cleaned up notification icon file: {}", icon);
                }
            });
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationUrgency {
    Low,
    Normal,
    Critical,
}
