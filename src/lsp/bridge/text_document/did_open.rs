//! Eager didOpen notification handling for bridge connections.
//!
//! This module provides eager opening of virtual documents on downstream
//! language servers when injection regions are detected during `did_open`
//! or `did_change` processing.

use super::super::pool::LanguageServerPool;

impl LanguageServerPool {
    /// Eagerly open virtual documents on a downstream server.
    ///
    /// For each injection region, builds a `VirtualDocumentUri` and calls
    /// `ensure_document_opened`. This sends `didOpen` notifications to the
    /// downstream server so it can start analyzing immediately, rather than
    /// waiting for the first user-initiated request.
    ///
    /// # Arguments
    /// * `server_name` - The server name (for connection lookup)
    /// * `server_config` - The server configuration (for spawning if needed)
    /// * `host_uri` - The host document URI (e.g., markdown file)
    /// * `host_uri_lsp` - The host URI in `ls_types::Uri` format
    /// * `injections` - List of (language, region_id, content) tuples
    ///
    /// # Error Handling
    /// Errors are logged at debug level and never propagated. This method is
    /// fire-and-forget — a failure to open one document does not affect others.
    pub(crate) async fn eager_open_virtual_documents(
        &self,
        server_name: &str,
        server_config: &crate::config::settings::BridgeServerConfig,
        host_uri: &url::Url,
        host_uri_lsp: &tower_lsp_server::ls_types::Uri,
        injections: Vec<(String, String, String)>,
    ) {
        todo!("Subtask 4: implement eager_open_virtual_documents")
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::pool::test_helpers::*;
    use super::super::super::pool::{ConnectionState, LanguageServerPool};
    use super::super::super::protocol::VirtualDocumentUri;

    /// Test that eager_open_virtual_documents marks virtual documents as opened.
    ///
    /// Given a ready server and injection data, calling eager_open_virtual_documents
    /// should result in each virtual document being marked as opened in DocumentTracker.
    #[tokio::test]
    async fn eager_open_marks_documents_as_opened() {
        let pool = LanguageServerPool::new();
        let config = devnull_config();
        let server_name = "test-server";

        // Pre-create a ready connection so eager_open_virtual_documents finds it
        let handle = create_handle_with_state(ConnectionState::Ready).await;
        pool.insert_connection(server_name, handle).await;

        let host_uri = test_host_uri("eager_open");
        let host_uri_lsp = url_to_uri(&host_uri);

        let injections = vec![
            (
                "lua".to_string(),
                TEST_ULID_LUA_0.to_string(),
                "print('hello')".to_string(),
            ),
            (
                "lua".to_string(),
                TEST_ULID_LUA_1.to_string(),
                "print('world')".to_string(),
            ),
        ];

        pool.eager_open_virtual_documents(
            server_name,
            &config,
            &host_uri,
            &host_uri_lsp,
            injections,
        )
        .await;

        // Verify both virtual documents are marked as opened
        let vuri_0 = VirtualDocumentUri::new(&host_uri_lsp, "lua", TEST_ULID_LUA_0);
        let vuri_1 = VirtualDocumentUri::new(&host_uri_lsp, "lua", TEST_ULID_LUA_1);

        assert!(
            pool.is_document_opened(&vuri_0),
            "First virtual document should be marked as opened"
        );
        assert!(
            pool.is_document_opened(&vuri_1),
            "Second virtual document should be marked as opened"
        );
    }

    /// Test that eager_open_virtual_documents is idempotent.
    ///
    /// Calling it twice with the same injections should not cause errors or
    /// duplicate didOpen notifications. The second call should be a no-op
    /// for already-opened documents.
    #[tokio::test]
    async fn eager_open_is_idempotent() {
        let pool = LanguageServerPool::new();
        let config = devnull_config();
        let server_name = "test-server";

        let handle = create_handle_with_state(ConnectionState::Ready).await;
        pool.insert_connection(server_name, handle).await;

        let host_uri = test_host_uri("idempotent");
        let host_uri_lsp = url_to_uri(&host_uri);

        let injections = vec![(
            "lua".to_string(),
            TEST_ULID_LUA_0.to_string(),
            "print('hello')".to_string(),
        )];

        // First call - should open the document
        pool.eager_open_virtual_documents(
            server_name,
            &config,
            &host_uri,
            &host_uri_lsp,
            injections.clone(),
        )
        .await;

        let vuri = VirtualDocumentUri::new(&host_uri_lsp, "lua", TEST_ULID_LUA_0);
        assert!(
            pool.is_document_opened(&vuri),
            "Should be opened after first call"
        );

        // Second call - should be a no-op (idempotent)
        pool.eager_open_virtual_documents(
            server_name,
            &config,
            &host_uri,
            &host_uri_lsp,
            injections,
        )
        .await;

        assert!(
            pool.is_document_opened(&vuri),
            "Should still be opened after second call"
        );
    }
}
