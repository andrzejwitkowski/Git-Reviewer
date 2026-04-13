pub struct EmbeddedAsset {
    pub content: &'static [u8],
    pub content_type: &'static str,
}

pub fn get(path: &str) -> Option<EmbeddedAsset> {
    match path {
        "index.html" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/index.html"),
            content_type: "text/html; charset=utf-8",
        }),
        "styles.css" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/styles.css"),
            content_type: "text/css; charset=utf-8",
        }),
        "app.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/app.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "core/review-shell-state.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/core/review-shell-state.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "core/file-tree.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/core/file-tree.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "use-cases/load-shell.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/use-cases/load-shell.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "use-cases/update-review-base.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/use-cases/update-review-base.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "use-cases/update-review-mode.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/use-cases/update-review-mode.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "use-cases/update-review-commit.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/use-cases/update-review-commit.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "use-cases/review-request.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/use-cases/review-request.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "use-cases/empty-review.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/use-cases/empty-review.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "use-cases/review-state-reset.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/use-cases/review-state-reset.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "ports/review-port.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/ports/review-port.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "adapters/http-review-port.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/adapters/http-review-port.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "adapters/dom-renderer.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/adapters/dom-renderer.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "adapters/dom-renderer-helpers.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/adapters/dom-renderer-helpers.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "core/comment-store.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/core/comment-store.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "core/review-comments.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/core/review-comments.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "use-cases/reload-shell.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/use-cases/reload-shell.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "use-cases/export-comments.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/use-cases/export-comments.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        "use-cases/watch-refresh.js" => Some(EmbeddedAsset {
            content: include_bytes!("../../../../web/use-cases/watch-refresh.js"),
            content_type: "application/javascript; charset=utf-8",
        }),
        _ => None,
    }
}
