# The Budi para Stellar service (server/): opens workspaces and proxies the
# agent's model calls. Runs the TypeScript directly with tsx, like `pnpm server`.
FROM node:22-slim
WORKDIR /app
RUN corepack enable
COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY tsconfig.json ./
COPY server ./server
COPY tenki ./tenki
ENV NODE_ENV=production
CMD ["pnpm", "exec", "tsx", "server/index.ts"]
