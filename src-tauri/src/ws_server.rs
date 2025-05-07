extern crate ws;

use std::sync::{ Arc, Mutex };
use ws::{ listen, Handler, Handshake, Message, Result as WsResult, Sender, CloseCode };
use tauri::{ command, AppHandle, Runtime, Emitter };
use tauri::ipc::InvokeError;
use tokio::task;
use get_if_addrs::get_if_addrs;

// 存储所有客户端的发送器
type Clients = Arc<Mutex<Vec<Sender>>>;

// 定义一个结构体来处理 WebSocket 事件
struct Server<R: Runtime> {
    out: Sender,
    clients: Clients,
    client_addr: Option<String>, // 新增字段用于存储客户端地址
    app_handle: AppHandle<R>, // 修改为 AppHandle
}

// 实现 Handler trait 来处理 WebSocket 事件
impl<R: Runtime> Handler for Server<R> {
    // 处理握手事件
    fn on_open(&mut self, handshake: Handshake) -> WsResult<()> {
        // 根据文档，remote_addr 返回 Result<Option<String>>
        if let Ok(Some(addr_str)) = handshake.remote_addr() {
            let full_msg = format!("新连接 {}", addr_str);
            println!("{}", full_msg);
            self.app_handle.emit("ws_server", full_msg.clone()).unwrap();
            self.client_addr = Some(addr_str); // 存储客户端地址
        } else {
            let msg = "新连接, ip err";
            println!("{}", msg);
            self.app_handle.emit("ws_server", msg).unwrap();
            self.client_addr = None;
        }
        self.clients.lock().unwrap().push(self.out.clone());
        Ok(())
    }

    // 处理接收到的消息
    fn on_message(&mut self, msg: Message) -> WsResult<()> {
        if let Some(addr) = &self.client_addr {
            let full_msg = format!("{}: {}", addr, msg);
            println!("{}", full_msg);
            self.app_handle.emit("ws_accept", full_msg.clone()).unwrap();
        } else {
            let full_msg = format!("ip err: {}", msg);
            println!("{}", full_msg);
            self.app_handle.emit("ws_accept", full_msg.clone()).unwrap();
        }
        return Ok(());
    }

    // 处理关闭事件
    fn on_close(&mut self, code: CloseCode, reason: &str) {
        if let Some(addr) = &self.client_addr {
            let msg = format!("连接关闭 {}", addr);
            println!("{}", msg);
            self.app_handle.emit("ws_server", msg.clone()).unwrap();
        } else {
            let msg = format!("连接已关闭, ip err: {:?} - {}", code, reason);
            println!("{}", msg);
            self.app_handle.emit("ws_server", msg).unwrap();
        }
        let mut clients = self.clients.lock().unwrap();
        if let Some(index) = clients.iter().position(|c| c == &self.out) {
            clients.remove(index);
        }
    }
}

// 全局的服务端句柄
static mut WS_SERVER_RUNNING: bool = false;
static mut CLIENTS: Option<Clients> = None;

#[command]
pub async fn start_ws_server<R: Runtime>(
    app_handle: AppHandle<R>,
    port: u16
) -> Result<String, InvokeError> {
    let clients: Clients = Arc::new(Mutex::new(Vec::new()));
    let address = format!("0.0.0.0:{}", port);
    // 获取所有网卡的 IP 地址
    match get_if_addrs() {
        Ok(interfaces) => {
            let mut ip_list: Vec<String> = Vec::new();
            for interface in interfaces {
                if let get_if_addrs::IfAddr::V4(ipv4) = interface.addr {
                    ip_list.push(format!("[{}]", ipv4.ip));
                }
            }
            let ip_msg = ip_list.join(" - ");
            println!("{}", ip_msg);
            app_handle.emit("ws_ips", ip_msg.clone()).unwrap();
        }
        Err(e) => {
            let error_msg = format!("获取网络信息失败: {:?}", e);
            println!("{}", error_msg);
            app_handle.emit("ws_ips", error_msg.clone()).unwrap();
        }
    }
    unsafe {
        if WS_SERVER_RUNNING {
            let error_msg = "服务已开启";
            println!("{}", error_msg);
            app_handle.emit("ws_server", error_msg.clone()).unwrap();
            return Err(InvokeError::from(error_msg.to_string()));
        }

        // 克隆 clients 以便可以在闭包和外部都使用
        let clients_for_handler = clients.clone();

        task::spawn(async move {
            // 启动监听前更新状态
            unsafe {
                WS_SERVER_RUNNING = true;
                CLIENTS = Some(clients.clone());
                let running_msg = format!("WS_SERVER_RUNNING: {}", WS_SERVER_RUNNING);
                println!("{}", running_msg);
                let clients_msg = format!("CLIENTS: {:?}", CLIENTS);
                println!("{}", clients_msg);
            }
            let listen_ok_msg = format!("服务已监听: {}", port);
            println!("{}", listen_ok_msg);
            app_handle.emit("ws_server", listen_ok_msg.clone()).unwrap();

            match
                listen(address, move |out| Server {
                    out,
                    clients: clients_for_handler.clone(), // 使用克隆的实例
                    client_addr: None, // 新增：初始化 client_addr 字段
                    app_handle: app_handle.clone(), // 传递 app_handle 到 Server 结构体
                })
            {
                Ok(_) => {
                    // 监听结束后更新状态
                    unsafe {
                        WS_SERVER_RUNNING = false;
                        CLIENTS = None;
                        let stopped_msg = "服务停止";
                        println!("{}", stopped_msg);
                        // app_handle.emit("ws_server", stopped_msg).unwrap();
                    }
                }
                Err(e) => {
                    let listen_error_msg = format!("监听出错: {:?}", e);
                    println!("{}", listen_error_msg);
                    // app_handle.emit("ws_server", listen_error_msg.clone()).unwrap();
                    // 监听出错后更新状态
                    unsafe {
                        WS_SERVER_RUNNING = false;
                        CLIENTS = None;
                    }
                }
            }
        });

        let return_msg = format!("return server ok: {}", port);
        println!("{}", return_msg);
        // app_handle.emit("ws_server", return_msg.clone()).unwrap();
        Ok(format!("WebSocket server started on port {}", port))
    }
}

// 服务端主动群发消息给所有客户端
#[command]
pub fn broadcast_message<R: Runtime>(
    app_handle: AppHandle<R>,
    message: String
) -> Result<String, InvokeError> {
    unsafe {
        if let Some(clients) = &CLIENTS {
            let clients_lock = clients.lock().unwrap();
            if clients_lock.is_empty() {
                let error_msg = "无客户端连接";
                println!("{}", error_msg);
                app_handle.emit("ws_server", error_msg.clone()).unwrap();
                return Err(InvokeError::from(error_msg.to_string()));
            }
            for client in clients_lock.iter() {
                if let Err(e) = client.send(Message::Text(message.clone())) {
                    let send_error_msg = format!("发送失败: {}", e);
                    println!("{}", send_error_msg);
                    app_handle.emit("ws_server", send_error_msg.clone()).unwrap();
                    return Err(InvokeError::from(send_error_msg.clone()));
                }
            }
            println!("{}", message);
            app_handle.emit("ws_send", message.clone()).unwrap();
            Ok("send ok".to_string())
        } else {
            let error_msg = "服务未运行";
            println!("{}", error_msg);
            app_handle.emit("ws_server", error_msg.clone()).unwrap();
            return Err(InvokeError::from(error_msg.to_string()));
        }
    }
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
