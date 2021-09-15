# Compoker

A complexity poker app built to learn Svelte and Actix.

## Get started

Install the dependencies...

```bash
cargo build
npm install
```

...then start [Rollup](https://rollupjs.org) and the server:

```bash
npm run dev & 
cargo run
```

Navigate to [localhost:8080](http://localhost:8080). You should see your app running. Edit a component file in `src`, save it, and reload the page to see your changes.

By default, the server will only respond to requests from localhost. To allow connections from other computers, edit the `sirv` commands in package.json to include the option `--host 0.0.0.0`.

Any changes to the server code will require you to stop, recompile and restart the server.

## Building and running in production mode

To create an optimised version of the app:

```bash
npm run build
cargo build --release
```
