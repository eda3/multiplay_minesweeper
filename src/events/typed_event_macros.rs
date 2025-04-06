/**
 * 型付きイベント定義用マクロ
 * 
 * イベントの定義と実装を簡略化し、タイプセーフなイベントシステムをサポートする
 * マクロのコレクション。
 */
use std::any::TypeId;
use std::collections::HashMap;
use serde::ser::{Serialize, Serializer, SerializeMap};
use serde_json::json;
use crate::events::EventData;

/// イベント構造体とTypedEventトレイトの実装を一度に定義するマクロ
/// 
/// # 使用例
/// ```
/// define_event! {
///     /// ゲーム開始イベント
///     pub struct GameStartEvent {
///         /// プレイヤー名
///         pub player_name: String,
///         /// 難易度
///         pub difficulty: u8,
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_event {
    ($event_type:ident, $( $field_name:ident: $field_type:ty ),*) => {
        #[derive(Debug, Clone)]
        pub struct $event_type {
            $(pub $field_name: $field_type,)*
            /// イベント発生時のタイムスタンプ（内部使用）
            #[doc(hidden)]
            pub _timestamp: u64,
        }

        impl $event_type {
            pub fn new($($field_name: $field_type,)*) -> Self {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                
                Self {
                    $($field_name,)*
                    _timestamp: now,
                }
            }
            
            /// タイムスタンプを指定してイベントを作成
            pub fn with_timestamp($($field_name: $field_type,)* timestamp: u64) -> Self {
                Self {
                    $($field_name,)*
                    _timestamp: timestamp,
                }
            }

            pub fn to_event_data(&self) -> Option<crate::events::event_data::EventData> {
                // JSONシリアライズを使用した基本実装
                match serde_json::to_string(self) {
                    Ok(json_string) => Some(crate::events::EventData::String(json_string)),
                    Err(_) => None,
                }
            }
        }

        impl crate::events::typed_event::TypedEvent for $event_type {
            fn event_type() -> &'static str {
                stringify!($event_type)
            }
            
            fn timestamp(&self) -> u64 {
                self._timestamp
            }
        }

        impl serde::Serialize for $event_type {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut map = serializer.serialize_map(None)?;
                $(
                    map.serialize_entry(stringify!($field_name), &self.$field_name)?;
                )*
                map.serialize_entry("_timestamp", &self._timestamp)?;
                map.end()
            }
        }
    };
}

/// イベントハンドラを簡潔に定義するマクロ
/// 
/// # 使用例
/// ```
/// event_handler! {
///     fn handle_game_start(event: &GameStartEvent, resources: &Resources) {
///         println!("ゲーム開始: {}, 難易度: {}", event.player_name, event.difficulty);
///     }
/// }
/// ```
#[macro_export]
macro_rules! event_handler {
    (
        fn $name:ident($event_param:ident: &$event_type:ty $(, $param_name:ident: $param_type:ty)*) $body:block
    ) => {
        pub fn $name($event_param: &$event_type $(, $param_name: $param_type)*) $body
        
        pub fn register_$name(event_bus: &$crate::events::typed_event_bus::TypedEventBus $(, $param_name: $param_type)*) -> $crate::events::typed_handler::TypedEventHandler<$event_type> {
            let handler = move |event: &$event_type| {
                $name(event $(, $param_name.clone())*)
            };
            
            event_bus.subscribe(
                stringify!($name),
                handler
            )
        }
    };
}

/// 型安全なイベント処理関数を定義するマクロ
/// 
/// # 使用例
/// ```
/// typed_event_processor! {
///     /// ゲーム開始処理
///     fn process_game_start(
///         event: &GameStartEvent,
///         resources: &mut Resources
///     ) -> Result<(), EventError> {
///         // イベント処理ロジック
///         Ok(())
///     }
/// }
/// ```
#[macro_export]
macro_rules! typed_event_processor {
    (
        $(#[$fn_meta:meta])*
        fn $name:ident(
            $event_param:ident: &$event_type:ty,
            $resources_param:ident: &mut $resources_type:ty
            $(, $param_name:ident: $param_type:ty)*
        ) -> $ret:ty $body:block
    ) => {
        $(#[$fn_meta])*
        pub fn $name(
            $event_param: &$event_type,
            $resources_param: &mut $resources_type
            $(, $param_name: $param_type)*
        ) -> $ret $body
        
        pub fn register_$name(
            event_bus: &$crate::events::typed_event_bus::TypedEventBus,
            $resources_param: std::sync::Arc<std::sync::RwLock<$resources_type>>
            $(, $param_name: $param_type)*
        ) -> $crate::events::typed_handler::TypedEventHandler<$event_type> {
            use std::sync::{Arc, RwLock};
            
            let resources_clone = $resources_param.clone();
            $(let $param_name = $param_name.clone();)*
            
            let handler = move |event: &$event_type| {
                if let Ok(mut resources) = resources_clone.write() {
                    let _ = $name(event, &mut *resources $(, $param_name.clone())*);
                }
            };
            
            event_bus.subscribe(
                stringify!($name),
                handler
            )
        }
    };
}

/// イベントを発行するヘルパーマクロ
/// 
/// # 使用例
/// ```
/// publish_event!(event_bus, GameStartEvent { player_name: "Player1".to_string(), difficulty: 2 });
/// ```
#[macro_export]
macro_rules! publish_event {
    ($event_bus:expr, $event_type:ident { $($field:ident: $value:expr),* $(,)? }) => {{
        let event = $event_type::new($($field: $value,)*);
        $event_bus.publish(event);
    }};
    
    // 優先度付きバージョン
    ($event_bus:expr, $event_type:ident { $($field:ident: $value:expr),* $(,)? }, $priority:expr) => {{
        let event = $event_type::new($($field: $value,)*);
        $event_bus.publish_with_priority(event, $priority);
    }};
}