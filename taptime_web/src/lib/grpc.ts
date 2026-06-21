import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { env } from "$env/dynamic/public";

const apiUrl = env.PUBLIC_API_URL ?? "http://127.0.0.1:50051";

export const transport = createGrpcWebTransport({
  baseUrl: apiUrl,
});
