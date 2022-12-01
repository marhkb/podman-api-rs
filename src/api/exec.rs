use crate::{
    conn::{tty, Headers, Payload},
    opts, Result, Stream, TryFutureExt, TryStreamExt, Value,
};

use containers_api::url;

impl_api_ty!(Exec => id);

impl Exec {
    api_doc! {
    Exec => StartLibpod
    |
    /// Starts a previously set up exec instance. If `detach` is true, this endpoint returns
    /// immediately after starting the command. Otherwise, it sets up an interactive session
    /// with the command.
    ///
    /// To create an exec instance use [`Container::create_exec`](crate::api::Container::create_exec).
    ///
    /// Examples:
    ///
    /// ```no_run
    /// async {
    ///     use podman_api::Podman;
    ///     use futures_util::StreamExt;
    ///     let podman = Podman::unix("/run/user/1000/podman/podman.sock");
    ///     let container = podman.containers().get("451b27c6b9d3");
    ///
    ///     let exec = container
    ///         .create_exec(
    ///             &podman_api::opts::ExecCreateOpts::builder()
    ///                 .command(["cat", "/some/path/in/container"])
    ///                 .build(),
    ///         )
    ///         .await
    ///         .unwrap();
    ///
    ///     let opts = Default::default();
    ///     let mut stream = exec.start(&opts);
    ///
    ///     while let Some(chunk) = stream.next().await {
    ///         println!("{:?}", chunk.unwrap());
    ///     }
    /// };
    /// ```
    pub async fn start<'exec>(
        &'exec self,
        opts: &'exec opts::ExecStartOpts,
    ) -> Result<tty::Multiplexer<'exec>>
    {
        let ep = format!("/libpod/exec/{}/start", &self.id);

        let payload = Payload::Json(
            opts.serialize()
                .map_err(|e| crate::conn::Error::Any(Box::new(e)))?,
        );
        self.podman
            .post_upgrade_stream(ep, payload)
            .await
            .map(|x| {
                // When the container allocates a TTY the stream doesn't come in the standard
                // Docker format but rather as a raw stream of bytes.
                // TODO: Somehow retrieve opts.params.get("tty")
                if true {
                    tty::Multiplexer::new(x, tty::decode_raw)
                } else {
                    tty::Multiplexer::new(x, tty::decode_chunk)
                }
            })
    }}

    api_doc! {
    Exec => InspectLibpod
    |
    /// Returns low-level information about an exec instance.
    ///
    /// Examples:
    ///
    /// ```no_run
    /// async {
    ///     use podman_api::Podman;
    ///     use futures_util::StreamExt;
    ///     let podman = Podman::unix("/run/user/1000/podman/podman.sock");
    ///     let container = podman.containers().get("451b27c6b9d3");
    ///
    ///     let exec = container
    ///         .create_exec(
    ///             &podman_api::opts::ExecCreateOpts::builder()
    ///                 .command(["cat", "/some/path/in/container"])
    ///                 .build(),
    ///         )
    ///         .await
    ///         .unwrap();
    ///
    ///     match exec.inspect().await {
    ///         Ok(info) => println!("{:?}", info),
    ///         Err(e) => eprintln!("{}", e)
    ///     }
    /// };
    /// ```
    pub async fn inspect(&self) -> Result<Value> {
        let ep = format!("/libpod/exec/{}/json", &self.id);
        self.podman.get_json(&ep).await
    }}

    api_doc! {
    Exec => ResizeLibpod
    |
    /// Resize the TTY session used by an exec instance. This endpoint only works if
    /// tty was specified as part of creating and starting the exec instance.
    ///
    /// Examples:
    ///
    /// ```no_run
    /// use futures_util::StreamExt;
    /// async {
    ///     use podman_api::Podman;
    ///     let podman = Podman::unix("/run/user/1000/podman/podman.sock");
    ///     let container = podman.containers().get("451b27c6b9d3");
    ///
    ///     let exec = container
    ///         .create_exec(
    ///             &podman_api::opts::ExecCreateOpts::builder()
    ///                 .command(["cat", "/some/path/in/container"])
    ///                 .build(),
    ///         )
    ///         .await
    ///         .unwrap();
    ///
    ///     if let Err(e) = exec.resize(1280, 720).await {
    ///         eprintln!("{}", e);
    ///     }
    /// };
    /// ```
    pub async fn resize(&self, width: usize, heigth: usize) -> Result<()> {
        let ep = url::construct_ep(
            format!("/libpod/exec/{}/resize", &self.id),
            Some(url::encoded_pairs([
                ("h", heigth.to_string()),
                ("w", width.to_string()),
            ])),
        );
        self.podman.post(&ep, Payload::None::<&str>, Headers::none()).await.map(|_| ())
    }}
}
