use alloy_json_rpc::{Response, ResponsePayload, SerializedRequest, SubId};
use alloy_primitives::B256;
use alloy_transport::{TransportError, TransportResult};
use std::fmt;
use tokio::sync::oneshot;

/// The outcome of fulfilling an in-flight request.
#[derive(Debug)]
pub enum RequestOutcome {
    /// Not a subscription request. Response was sent to the original caller.
    NonSubscription,
    /// Subscription request succeeded. Contains the server ID and in-flight for registration.
    SubscriptionSuccess(SubId, InFlight),
    /// Subscription request failed. Contains the local ID for cleanup.
    SubscriptionFailed(B256),
}

/// An in-flight JSON-RPC request.
///
/// This struct contains the request that was sent, as well as a channel to
/// receive the response on.
pub struct InFlight {
    /// The request
    pub request: SerializedRequest,

    /// The number of items to buffer in the subscription channel.
    pub channel_size: usize,

    /// The channel to send the response on.
    pub tx: oneshot::Sender<TransportResult<Response>>,

    /// Local ID for subscription requests, computed from params hash.
    pub local_id: B256,
}

impl fmt::Debug for InFlight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InFlight")
            .field("request", &self.request)
            .field("channel_size", &self.channel_size)
            .field("tx_is_closed", &self.tx.is_closed())
            .finish()
    }
}

impl InFlight {
    /// Create a new in-flight request.
    pub fn new(
        request: SerializedRequest,
        channel_size: usize,
    ) -> (Self, oneshot::Receiver<TransportResult<Response>>) {
        let (tx, rx) = oneshot::channel();
        let local_id = request.params_hash();

        (Self { request, channel_size, tx, local_id }, rx)
    }

    /// Check if the request is a subscription.
    pub fn is_subscription(&self) -> bool {
        self.request.is_subscription()
    }

    /// Get a reference to the serialized request.
    ///
    /// This is used to (re-)send the request over the transport.
    pub const fn request(&self) -> &SerializedRequest {
        &self.request
    }

    /// Fulfill the request with a response. This consumes the in-flight request.
    pub fn fulfill(self, resp: Response) -> RequestOutcome {
        if self.is_subscription() {
            let local_id = self.local_id;
            if let ResponsePayload::Success(val) = resp.payload {
                let sub_id: serde_json::Result<SubId> = serde_json::from_str(val.get());
                return match sub_id {
                    Ok(alias) => RequestOutcome::SubscriptionSuccess(alias, self),
                    Err(e) => {
                        let _ = self.tx.send(Err(TransportError::deser_err(e, val.get())));
                        RequestOutcome::SubscriptionFailed(local_id)
                    }
                };
            }
            let _ = self.tx.send(Ok(resp));
            return RequestOutcome::SubscriptionFailed(local_id);
        }

        let _ = self.tx.send(Ok(resp));
        RequestOutcome::NonSubscription
    }
}
