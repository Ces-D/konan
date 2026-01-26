FROM node:25-slim AS base
ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
ENV CI=true
RUN npm install --global --force corepack@latest
RUN corepack enable pnpm
COPY . ./app
WORKDIR /app

FROM base AS prod-deps
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --prod --frozen-lockfile

FROM base AS build
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build

FROM gcr.io/distroless/nodejs24-debian13 AS runner
WORKDIR /app
COPY --from=prod-deps /app/node_modules /app/node_modules
COPY --from=build /app/build /app/build
COPY --from=build /app/package.json /app/package.json
EXPOSE 3000
CMD [ "/app/build/index.js" ]
