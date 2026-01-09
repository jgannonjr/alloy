use crate::managers::{InFlight, RequestOutcome};
use alloy_json_rpc::{Id, Response};
use alloy_primitives::map::HashMap;

/// Manages in-flight requests.
#[derive(Debug, Default)]
pub(crate) struct RequestManager {
    reqs: HashMap<Id, InFlight>,
}

impl RequestManager {
    /// Get the number of in-flight requests.
    pub(crate) fn len(&self) -> usize {
        self.reqs.len()
    }

    /// Get an iterator over the in-flight requests.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&Id, &InFlight)> {
        self.reqs.iter()
    }

    /// Insert a new in-flight request.
    pub(crate) fn insert(&mut self, in_flight: InFlight) {
        self.reqs.insert(in_flight.request.id().clone(), in_flight);
    }

    /// Handle a response by sending the payload to the waiter.
    ///
    /// Returns `None` if the response ID doesn't match any in-flight request.
    pub(crate) fn handle_response(&mut self, resp: Response) -> Option<RequestOutcome> {
        self.reqs.remove(&resp.id).map(|in_flight| in_flight.fulfill(resp))
    }
}
