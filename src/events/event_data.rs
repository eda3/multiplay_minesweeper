/**
 * イベントデータ型
 * 
 * イベントデータの標準的な表現を提供する列挙型。
 * 各イベントタイプからの変換先として使用される。
 */

/// イベントデータの標準表現
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EventData {
    /// 整数値
    Integer(i64),
    /// 浮動小数点値
    Float(f64),
    /// 文字列値
    String(String),
    /// バイナリデータ
    Binary(Vec<u8>),
    /// JSONシリアライズされたデータ
    Json(String),
    /// 複合データ（キー・バリューペア）
    Map(std::collections::HashMap<String, EventData>),
    /// 配列データ
    Array(Vec<EventData>),
    /// ブール値
    Boolean(bool),
    /// NULL値
    Null,
} 