/**
 * レンダリングシステム
 * 
 * ゲームの描画を担当するシステム
 */
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;

use crate::resources::RenderResource;
use crate::resources::{BoardResource, CellState};
use crate::resources::{GameStateResource, GamePhase};
use crate::resources::PlayerStateResource;
use crate::resources::TimeResource;

// 色の定義
const COLOR_BACKGROUND: &str = "#f0f0f0";
const COLOR_GRID: &str = "#bbbbbb";
const COLOR_REVEALED: &str = "#dddddd";
const COLOR_HIDDEN: &str = "#bbbbbb";
const COLOR_HOVER: &str = "#aaaaaa";
const COLOR_MINE: &str = "#000000";
const COLOR_FLAG: &str = "#ff0000";
const COLOR_TEXT: &str = "#000000";
const COLOR_TEXT_SHADOW: &str = "#ffffff";
const COLOR_GAME_OVER: &str = "rgba(0, 0, 0, 0.5)";
const COLOR_BUTTON: &str = "#4CAF50";
const COLOR_BUTTON_HOVER: &str = "#45a049";
const COLOR_BUTTON_TEXT: &str = "#ffffff";

// フォントの定義
const FONT_NORMAL: &str = "16px Arial";
const FONT_TITLE: &str = "bold 24px Arial";
const FONT_LARGE: &str = "bold 36px Arial";
const FONT_SMALL: &str = "12px Arial";

/// レンダリングシステム関数
pub fn render_system(resources: &[Rc<RefCell<dyn Any>>]) {
    // RenderResourceのRcを探す
    let render_rc_option = resources.iter()
        .find(|r| r.borrow().is::<RenderResource>());
    
    // BoardResourceのRcを探す
    let board_rc_option = resources.iter()
        .find(|r| r.borrow().is::<BoardResource>());
    
    // GameStateResourceのRcを探す
    let game_rc_option = resources.iter()
        .find(|r| r.borrow().is::<GameStateResource>());
    
    // PlayerStateResourceのRcを探す
    let player_rc_option = resources.iter()
        .find(|r| r.borrow().is::<PlayerStateResource>());
    
    // TimeResourceのRcを探す
    let time_rc_option = resources.iter()
        .find(|r| r.borrow().is::<TimeResource>());
    
    // 必要なリソースが見つからない場合は何もしない
    if render_rc_option.is_none() || game_rc_option.is_none() {
        return;
    }
    
    // レンダリング処理
    {
        let render = render_rc_option.unwrap().borrow();
        let render = render.downcast_ref::<RenderResource>().unwrap();
        let context = render.get_context();
        
        let game = game_rc_option.unwrap().borrow();
        let game = game.downcast_ref::<GameStateResource>().unwrap();
        
        // キャンバスをクリア
        clear_canvas(&context, render);
        
        // ゲームフェーズに応じたレンダリング
        match &game.phase {
            GamePhase::StartScreen => render_start_screen(&context, render),
            GamePhase::Loading => render_loading_screen(&context, render),
            GamePhase::Playing => {
                // ゲームボードとUIをレンダリング
                if let Some(board_rc) = board_rc_option {
                    let board = board_rc.borrow();
                    let board = board.downcast_ref::<BoardResource>().unwrap();
                    render_game_board(&context, render, board);
                    
                    // ゲームUIをレンダリング
                    if let Some(time_rc) = time_rc_option {
                        if let Some(player_rc) = player_rc_option {
                            let time = time_rc.borrow();
                            let time = time.downcast_ref::<TimeResource>().unwrap();
                            
                            let player = player_rc.borrow();
                            let player = player.downcast_ref::<PlayerStateResource>().unwrap();
                            
                            render_game_ui(&context, render, board, game, player, time);
                        }
                    }
                }
            },
            GamePhase::Paused => {
                // ゲームボードとポーズ画面をレンダリング
                if let Some(board_rc) = board_rc_option {
                    let board = board_rc.borrow();
                    let board = board.downcast_ref::<BoardResource>().unwrap();
                    render_game_board(&context, render, board);
                    render_pause_screen(&context, render);
                }
            },
            GamePhase::GameOver { win, score, time } => {
                // ゲームボードとゲームオーバー画面をレンダリング
                if let Some(board_rc) = board_rc_option {
                    let board = board_rc.borrow();
                    let board = board.downcast_ref::<BoardResource>().unwrap();
                    render_game_board(&context, render, board);
                    render_game_over_screen(&context, render, *win, *score, *time);
                }
            },
        }
    }
}

/// キャンバスをクリア
fn clear_canvas(context: &CanvasRenderingContext2d, render: &RenderResource) {
    let (width, height) = render.get_canvas_size();
    
    // 背景をクリア
    context.set_fill_style(&COLOR_BACKGROUND.into());
    context.fill_rect(0.0, 0.0, width as f64, height as f64);
}

/// スタート画面をレンダリング
fn render_start_screen(context: &CanvasRenderingContext2d, render: &RenderResource) {
    let (width, height) = render.get_canvas_size();
    let center_x = (width / 2) as f64;
    let center_y = (height / 2) as f64;
    
    // タイトル
    context.set_font(FONT_LARGE);
    context.set_text_align("center");
    context.set_text_baseline("middle");
    context.set_fill_style(&COLOR_TEXT.into());
    context.fill_text("マインスイーパー", center_x, center_y - 100.0).unwrap();
    
    // 説明テキスト
    context.set_font(FONT_NORMAL);
    context.fill_text("スペースキーを押してスタート", center_x, center_y).unwrap();
    
    // 難易度選択テキスト
    context.set_font(FONT_SMALL);
    context.fill_text("難易度を選択: 1 = 初級, 2 = 中級, 3 = 上級", center_x, center_y + 50.0).unwrap();
}

/// ロード画面をレンダリング
fn render_loading_screen(context: &CanvasRenderingContext2d, render: &RenderResource) {
    let (width, height) = render.get_canvas_size();
    let center_x = (width / 2) as f64;
    let center_y = (height / 2) as f64;
    
    // ローディングテキスト
    context.set_font(FONT_TITLE);
    context.set_text_align("center");
    context.set_text_baseline("middle");
    context.set_fill_style(&COLOR_TEXT.into());
    context.fill_text("読み込み中...", center_x, center_y).unwrap();
}

/// ゲームボードをレンダリング
fn render_game_board(context: &CanvasRenderingContext2d, render: &RenderResource, board: &BoardResource) {
    let cell_size = board.config.cell_size as f64;
    let board_width = board.config.width;
    let board_height = board.config.height;
    
    // ボードの中央配置のためのオフセット計算
    let (canvas_width, canvas_height) = render.get_canvas_size();
    let offset_x = ((canvas_width as usize - board.config.board_width_px()) / 2) as f64;
    let offset_y = ((canvas_height as usize - board.config.board_height_px()) / 2) as f64;
    
    // グリッドの描画
    context.set_stroke_style(&COLOR_GRID.into());
    context.set_line_width(1.0);
    
    // セルの描画
    for y in 0..board_height {
        for x in 0..board_width {
            let index = y * board_width + x;
            let cell = &board.cells[index];
            
            let cell_x = offset_x + (x as f64 * cell_size);
            let cell_y = offset_y + (y as f64 * cell_size);
            
            // セルの背景色を設定
            match cell.state {
                CellState::Hidden => {
                    context.set_fill_style(&COLOR_HIDDEN.into());
                },
                CellState::Revealed => {
                    context.set_fill_style(&COLOR_REVEALED.into());
                },
                CellState::Flagged => {
                    context.set_fill_style(&COLOR_HIDDEN.into());
                },
                CellState::Exploded => {
                    context.set_fill_style(&COLOR_MINE.into());
                },
            }
            
            // セルを描画
            context.fill_rect(cell_x, cell_y, cell_size, cell_size);
            context.stroke_rect(cell_x, cell_y, cell_size, cell_size);
            
            // セルの内容を描画
            match cell.state {
                CellState::Revealed => {
                    if !cell.is_mine && cell.adjacent_mines > 0 {
                        // 周囲の地雷数を表示
                        let number_color = get_number_color(cell.adjacent_mines);
                        
                        context.set_font(FONT_TITLE);
                        context.set_text_align("center");
                        context.set_text_baseline("middle");
                        context.set_fill_style(&number_color.into());
                        
                        context.fill_text(
                            &cell.adjacent_mines.to_string(),
                            cell_x + cell_size / 2.0,
                            cell_y + cell_size / 2.0,
                        ).unwrap();
                    }
                },
                CellState::Flagged => {
                    // フラグを描画
                    draw_flag(context, cell_x, cell_y, cell_size);
                },
                CellState::Exploded => {
                    // 爆発した地雷を描画
                    draw_mine(context, cell_x, cell_y, cell_size, true);
                },
                _ => {}
            }
            
            // 地雷を表示（ゲームオーバー時）
            if cell.is_mine && cell.state == CellState::Revealed {
                draw_mine(context, cell_x, cell_y, cell_size, false);
            }
        }
    }
}

/// ゲームUIをレンダリング
fn render_game_ui(
    context: &CanvasRenderingContext2d,
    render: &RenderResource,
    board: &BoardResource,
    game: &GameStateResource,
    player: &PlayerStateResource,
    time: &TimeResource
) {
    let (width, _height) = render.get_canvas_size();
    
    // UIの背景
    context.set_fill_style(&"rgba(240, 240, 240, 0.8)".into());
    context.fill_rect(0.0, 0.0, width as f64, 40.0);
    
    // 残りの地雷数
    context.set_font(FONT_NORMAL);
    context.set_text_align("left");
    context.set_text_baseline("middle");
    context.set_fill_style(&COLOR_TEXT.into());
    
    context.fill_text(
        &format!("地雷: {}", board.remaining_mines),
        20.0,
        20.0,
    ).unwrap();
    
    // 経過時間
    let seconds = (game.elapsed_time / 1000.0) as u32;
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    
    context.set_text_align("right");
    context.fill_text(
        &format!("時間: {:02}:{:02}", minutes, seconds),
        width as f64 - 20.0,
        20.0,
    ).unwrap();
    
    // プレイヤー情報とスコア（中央）
    context.set_text_align("center");
    context.fill_text(
        &format!("プレイヤー: {} | スコア: {}", player.player_name, game.score),
        width as f64 / 2.0,
        20.0,
    ).unwrap();
}

/// ポーズ画面をレンダリング
fn render_pause_screen(context: &CanvasRenderingContext2d, render: &RenderResource) {
    let (width, height) = render.get_canvas_size();
    
    // 半透明の背景
    context.set_fill_style(&"rgba(0, 0, 0, 0.5)".into());
    context.fill_rect(0.0, 0.0, width as f64, height as f64);
    
    // ポーズテキスト
    context.set_font(FONT_LARGE);
    context.set_text_align("center");
    context.set_text_baseline("middle");
    context.set_fill_style(&"#ffffff".into());
    
    context.fill_text(
        "ポーズ中",
        width as f64 / 2.0,
        height as f64 / 2.0 - 50.0,
    ).unwrap();
    
    // 指示テキスト
    context.set_font(FONT_NORMAL);
    context.fill_text(
        "ESCキーまたはスペースキーで再開",
        width as f64 / 2.0,
        height as f64 / 2.0,
    ).unwrap();
    
    context.fill_text(
        "Rキーでリセット",
        width as f64 / 2.0,
        height as f64 / 2.0 + 30.0,
    ).unwrap();
}

/// ゲームオーバー画面をレンダリング
fn render_game_over_screen(
    context: &CanvasRenderingContext2d,
    render: &RenderResource,
    win: bool,
    score: u32,
    time: std::time::Duration,
) {
    let (width, height) = render.get_canvas_size();
    
    // 半透明の背景
    context.set_fill_style(&COLOR_GAME_OVER.into());
    context.fill_rect(0.0, 0.0, width as f64, height as f64);
    
    // ゲームオーバーテキスト
    context.set_font(FONT_LARGE);
    context.set_text_align("center");
    context.set_text_baseline("middle");
    
    if win {
        context.set_fill_style(&"#00ff00".into());
        context.fill_text(
            "クリア！",
            width as f64 / 2.0,
            height as f64 / 2.0 - 70.0,
        ).unwrap();
    } else {
        context.set_fill_style(&"#ff0000".into());
        context.fill_text(
            "ゲームオーバー",
            width as f64 / 2.0,
            height as f64 / 2.0 - 70.0,
        ).unwrap();
    }
    
    // スコアとタイム
    context.set_font(FONT_NORMAL);
    context.set_fill_style(&"#ffffff".into());
    
    let seconds = time.as_secs();
    let minutes = seconds / 60;
    let seconds = seconds % 60;
    
    context.fill_text(
        &format!("スコア: {}", score),
        width as f64 / 2.0,
        height as f64 / 2.0 - 30.0,
    ).unwrap();
    
    context.fill_text(
        &format!("タイム: {:02}:{:02}", minutes, seconds),
        width as f64 / 2.0,
        height as f64 / 2.0,
    ).unwrap();
    
    // 指示テキスト
    context.set_font(FONT_NORMAL);
    context.fill_text(
        "スペースキーまたはRキーで再開",
        width as f64 / 2.0,
        height as f64 / 2.0 + 40.0,
    ).unwrap();
    
    context.fill_text(
        "ESCキーでメニューに戻る",
        width as f64 / 2.0,
        height as f64 / 2.0 + 70.0,
    ).unwrap();
}

/// 周囲の地雷数に応じた色を取得
fn get_number_color(mines: u8) -> &'static str {
    match mines {
        1 => "#0000FF", // 青
        2 => "#008000", // 緑
        3 => "#FF0000", // 赤
        4 => "#000080", // 紺
        5 => "#800000", // 茶
        6 => "#008080", // シアン
        7 => "#000000", // 黒
        8 => "#808080", // グレー
        _ => "#000000", // デフォルト
    }
}

/// フラグを描画
fn draw_flag(context: &CanvasRenderingContext2d, x: f64, y: f64, size: f64) {
    let flag_pole_x = x + size * 0.7;
    let flag_pole_y1 = y + size * 0.3;
    let flag_pole_y2 = y + size * 0.8;
    
    // 旗竿
    context.begin_path();
    context.set_line_width(2.0);
    context.set_stroke_style(&"#000000".into());
    context.move_to(flag_pole_x, flag_pole_y1);
    context.line_to(flag_pole_x, flag_pole_y2);
    context.stroke();
    
    // 旗
    context.begin_path();
    context.set_fill_style(&COLOR_FLAG.into());
    context.move_to(flag_pole_x, flag_pole_y1);
    context.line_to(flag_pole_x - size * 0.2, flag_pole_y1 + size * 0.15);
    context.line_to(flag_pole_x, flag_pole_y1 + size * 0.3);
    context.fill();
}

/// 地雷を描画
fn draw_mine(context: &CanvasRenderingContext2d, x: f64, y: f64, size: f64, exploded: bool) {
    let center_x = x + size / 2.0;
    let center_y = y + size / 2.0;
    let radius = size * 0.3;
    
    // 爆発エフェクト（爆発した場合）
    if exploded {
        context.set_fill_style(&"#ff0000".into());
        context.begin_path();
        context.arc(center_x, center_y, radius * 1.5, 0.0, std::f64::consts::PI * 2.0).unwrap();
        context.fill();
    }
    
    // 地雷の本体
    context.set_fill_style(&COLOR_MINE.into());
    context.begin_path();
    context.arc(center_x, center_y, radius, 0.0, std::f64::consts::PI * 2.0).unwrap();
    context.fill();
    
    // トゲ
    context.set_stroke_style(&COLOR_MINE.into());
    context.set_line_width(2.0);
    
    for i in 0..8 {
        let angle = (i as f64) * std::f64::consts::PI / 4.0;
        let spike_length = radius * 0.8;
        
        context.begin_path();
        context.move_to(
            center_x + radius * angle.cos(),
            center_y + radius * angle.sin(),
        );
        context.line_to(
            center_x + (radius + spike_length) * angle.cos(),
            center_y + (radius + spike_length) * angle.sin(),
        );
        context.stroke();
    }
    
    // 光沢
    context.set_fill_style(&"#ffffff".into());
    context.begin_path();
    context.arc(
        center_x - radius * 0.3,
        center_y - radius * 0.3,
        radius * 0.2,
        0.0,
        std::f64::consts::PI * 2.0,
    ).unwrap();
    context.fill();
} 