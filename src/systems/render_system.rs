/**
 * 描画システム
 * 
 * エンティティの描画を担当するシステム
 */
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlImageElement};

use crate::entities::EntityManager;
use crate::systems::system_registry::DeltaTime;
use crate::components::{Position, Renderable, Board};
use crate::resources::{RenderResource, BoardResource};

/// 描画システム - 画面描画を担当
pub fn render_system(
    entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    _delta_time: DeltaTime,
) -> Result<(), JsValue> {
    // 描画リソースを取得
    let render_resource = resources.get("render").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<RenderResource>().map(|r| r.clone())
    });
    
    if let Some(render) = render_resource {
        let context = render.context.clone();
        
        // 画面をクリア
        context.set_fill_style(&JsValue::from_str("#333333"));
        context.fill_rect(0.0, 0.0, render.canvas_width, render.canvas_height);
        
        // ボードの描画
        draw_board(entity_manager, resources, &context)?;
        
        // 描画可能コンポーネントを持つエンティティの描画
        draw_renderables(entity_manager, &context)?;
    }
    
    Ok(())
}

/// ボードの描画
fn draw_board(
    entity_manager: &mut EntityManager,
    resources: &mut HashMap<&'static str, Rc<RefCell<dyn std::any::Any>>>,
    context: &CanvasRenderingContext2d,
) -> Result<(), JsValue> {
    // ボードリソースを取得
    let board_resource = resources.get("board").and_then(|res| {
        res.clone().borrow_mut().downcast_mut::<BoardResource>().map(|r| r.clone())
    });
    
    if let Some(board) = board_resource {
        // ボードコンポーネントを持つエンティティを検索
        let board_entities = entity_manager.find_entities_with_component::<Board>();
        
        if let Some(board_entity) = board_entities.first() {
            if let Some(board_comp) = entity_manager.get_component::<Board>(*board_entity) {
                // セルサイズを取得
                let cell_size = board.cell_size;
                
                // ボードのセルを描画
                for y in 0..board_comp.height {
                    for x in 0..board_comp.width {
                        let idx = y * board_comp.width + x;
                        let cell_x = x as f64 * cell_size;
                        let cell_y = y as f64 * cell_size;
                        
                        // セルの背景を描画
                        if board_comp.revealed[idx] {
                            // 公開済みセル
                            context.set_fill_style(&JsValue::from_str("#DDDDDD"));
                        } else {
                            // 未公開セル
                            context.set_fill_style(&JsValue::from_str("#AAAAAA"));
                        }
                        
                        // セル描画（内側に1pxの枠を持たせる）
                        context.fill_rect(
                            cell_x + 1.0,
                            cell_y + 1.0,
                            cell_size - 2.0,
                            cell_size - 2.0,
                        );
                        
                        // セルの内容を描画
                        if board_comp.revealed[idx] {
                            // 公開済みセルの内容
                            let cell_value = board_comp.cells[idx];
                            
                            if cell_value == -1 {
                                // 地雷
                                context.set_fill_style(&JsValue::from_str("#FF0000"));
                                context.begin_path();
                                context.arc(
                                    cell_x + cell_size / 2.0,
                                    cell_y + cell_size / 2.0,
                                    cell_size / 4.0,
                                    0.0,
                                    std::f64::consts::PI * 2.0,
                                )?;
                                context.fill();
                            } else if cell_value > 0 {
                                // 数字
                                let colors = [
                                    "#0000FF", "#008000", "#FF0000", "#000080",
                                    "#800000", "#008080", "#000000", "#808080"
                                ];
                                
                                context.set_font(&format!("bold {}px Arial", cell_size / 2.0));
                                context.set_text_align("center");
                                context.set_text_baseline("middle");
                                context.set_fill_style(&JsValue::from_str(colors[(cell_value - 1) as usize]));
                                
                                context.fill_text(
                                    &cell_value.to_string(),
                                    cell_x + cell_size / 2.0,
                                    cell_y + cell_size / 2.0,
                                )?;
                            }
                        } else if board_comp.flagged[idx] {
                            // フラグ
                            context.set_fill_style(&JsValue::from_str("#FF8000"));
                            context.begin_path();
                            
                            // 三角形の旗を描画
                            let flag_x = cell_x + cell_size / 4.0;
                            let flag_y = cell_y + cell_size / 4.0;
                            let flag_size = cell_size / 2.0;
                            
                            context.move_to(flag_x, flag_y);
                            context.line_to(flag_x, flag_y + flag_size);
                            context.line_to(flag_x + flag_size, flag_y + flag_size / 2.0);
                            context.close_path();
                            context.fill();
                        }
                    }
                }
                
                // ゲームオーバーまたは勝利時の表示
                if board_comp.game_over || board_comp.game_won {
                    let message = if board_comp.game_won {
                        "You Win!"
                    } else {
                        "Game Over"
                    };
                    
                    // 半透明の背景
                    context.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.5)"));
                    context.fill_rect(
                        0.0,
                        0.0,
                        board_comp.width as f64 * cell_size,
                        board_comp.height as f64 * cell_size,
                    );
                    
                    // メッセージ
                    context.set_font(&format!("bold {}px Arial", cell_size * 1.5));
                    context.set_text_align("center");
                    context.set_text_baseline("middle");
                    context.set_fill_style(&JsValue::from_str(if board_comp.game_won { "#00FF00" } else { "#FF0000" }));
                    
                    context.fill_text(
                        message,
                        (board_comp.width as f64 * cell_size) / 2.0,
                        (board_comp.height as f64 * cell_size) / 2.0,
                    )?;
                }
            }
        }
    }
    
    Ok(())
}

/// 描画可能エンティティの描画
fn draw_renderables(
    entity_manager: &mut EntityManager,
    context: &CanvasRenderingContext2d,
) -> Result<(), JsValue> {
    // 描画コンポーネントと位置コンポーネントを持つエンティティを検索
    let renderable_entities = entity_manager.find_entities_with_components::<(Renderable, Position)>();
    
    // 描画の優先度でソート
    let mut entities_with_priority: Vec<(usize, i32)> = renderable_entities
        .iter()
        .map(|&entity_id| {
            let priority = entity_manager
                .get_component::<Renderable>(entity_id)
                .map(|r| r.z_index)
                .unwrap_or(0);
            (entity_id, priority)
        })
        .collect();
    
    entities_with_priority.sort_by_key(|&(_, priority)| priority);
    
    // 各エンティティを描画
    for (entity_id, _) in entities_with_priority {
        if let (Some(renderable), Some(position)) = (
            entity_manager.get_component::<Renderable>(entity_id),
            entity_manager.get_component::<Position>(entity_id),
        ) {
            match &renderable.render_type {
                // 画像描画
                crate::components::RenderType::Image(image_ref) => {
                    let image: &HtmlImageElement = image_ref.as_ref();
                    context.draw_image_with_html_image_element(
                        image,
                        position.x - (image.width() as f64 / 2.0),
                        position.y - (image.height() as f64 / 2.0),
                    )?;
                },
                
                // 円描画
                crate::components::RenderType::Circle { radius, color } => {
                    context.set_fill_style(&JsValue::from_str(color));
                    context.begin_path();
                    context.arc(
                        position.x,
                        position.y,
                        *radius,
                        0.0,
                        std::f64::consts::PI * 2.0,
                    )?;
                    context.fill();
                },
                
                // テキスト描画
                crate::components::RenderType::Text { text, font, color } => {
                    context.set_fill_style(&JsValue::from_str(color));
                    context.set_font(font);
                    context.set_text_align("center");
                    context.set_text_baseline("middle");
                    context.fill_text(text, position.x, position.y)?;
                },
                
                // 矩形描画
                crate::components::RenderType::Rectangle { width, height, color } => {
                    context.set_fill_style(&JsValue::from_str(color));
                    context.fill_rect(
                        position.x - (width / 2.0),
                        position.y - (height / 2.0),
                        *width,
                        *height,
                    );
                },
            }
        }
    }
    
    Ok(())
} 