use actix::*;
use actix_web::{HttpRequest, Result};
use crate::channel;
use crate::model::db::ConnDsl;

pub struct AppState {
    pub db: Addr<ConnDsl>,
    pub reset_sender: channel::Sender<String>,
}

pub fn home(_req: &HttpRequest<AppState>) -> Result<&'static str> {
    Ok("Available api's \n \
        GET /nodes \n \
        GET /nodes/<node_id> \n \
        POST /nodes <node json payload> \n \
        PUT /nodes <node json payload> \n \
        DELETE /nodes <node json payload> \n \
        POST /reboot_node/<node_id>")
}
