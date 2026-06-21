FROM node:22-bookworm-slim AS deps

WORKDIR /app

COPY taptime_proto/package.json taptime_proto/package-lock.json ./taptime_proto/
COPY taptime_proto/src ./taptime_proto/src
COPY taptime_web/package.json taptime_web/package-lock.json ./taptime_web/

WORKDIR /app/taptime_proto
RUN npm ci --omit=dev

WORKDIR /app/taptime_web
RUN npm ci

FROM deps AS builder

WORKDIR /app
COPY taptime_proto ./taptime_proto
COPY taptime_web ./taptime_web

WORKDIR /app/taptime_web
RUN npx svelte-kit sync && npm run build
RUN npm prune --omit=dev

FROM node:22-bookworm-slim AS runtime

ENV NODE_ENV=production
ENV HOST=0.0.0.0
ENV PORT=3000

WORKDIR /app/taptime_web

COPY --from=builder --chown=node:node /app/taptime_web/build ./build
COPY --from=builder --chown=node:node /app/taptime_web/package.json ./package.json
COPY --from=builder --chown=node:node /app/taptime_web/node_modules ./node_modules
COPY --from=builder --chown=node:node /app/taptime_proto /app/taptime_proto

USER node

EXPOSE 3000

CMD ["node", "build"]
