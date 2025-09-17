use std::error::Error;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;

use crate::clients::net_config::LiteEndpoint;
use crate::errors::TonError;
use adnl::AdnlPeer;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tower::multiplex::Client;
use ton_liteapi::layers::{WrapMessagesLayer, WrapService};
use ton_liteapi::peer::LitePeer;
use ton_liteapi::tl::adnl::Message;
use ton_liteapi::tl::request::WrappedRequest;
use ton_liteapi::tl::response::Response;
use tower::{Service, ServiceBuilder, ServiceExt};

type ConnService = WrapService<Client<LitePeer<AdnlPeer<TcpStream>>, Box<dyn Error + Sync + Send>, Message>>;

pub(super) struct Connection {
    public: Vec<u8>,
    addr: SocketAddrV4,
    conn_timeout: Duration,
    service: Option<ConnService>,
}

impl Connection {
    pub(super) fn new(endpoint: LiteEndpoint, conn_timeout: Duration) -> Result<Self, TonError> {
        let LiteEndpoint { ip, port, id } = endpoint;
        let ip_addr = Ipv4Addr::from(ip as u32);
        let public = BASE64_STANDARD.decode(id.key)?;
        let addr = SocketAddrV4::new(ip_addr, port);
        let conn = Self {
            public,
            addr,
            conn_timeout,
            service: None,
        };
        Ok(conn)
    }

    pub(super) async fn exec(&mut self, req: WrappedRequest, req_timeout: Duration) -> Result<Response, TonError> {
        let ready_service = self.connect().await?.ready().await?;
        Ok(timeout(req_timeout, ready_service.call(req)).await??)
    }

    pub(super) async fn connect(&mut self) -> Result<&mut ConnService, TonError> {
        if self.service.is_none() {
            let adnl = timeout(self.conn_timeout, AdnlPeer::connect(&self.public, self.addr)).await??;

            let lite = LitePeer::new(adnl);
            let service = ServiceBuilder::new().layer(WrapMessagesLayer).service(Client::<
                _,
                Box<dyn Error + Send + Sync + 'static>,
                _,
            >::new(lite));
            self.service = Some(service);
        }
        Ok(self.service.as_mut().unwrap()) // unwrap is safe: we initialized it in branch above
    }
}
