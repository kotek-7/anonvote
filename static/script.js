//@ts-check
import init, { greet } from "./pkg/client.js";

init().then(() => {
  greet("WebAssembly");
});
