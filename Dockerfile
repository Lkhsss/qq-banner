FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY . .


# 关键：针对musl目标进行静态编译
RUN cargo build --release --target x86_64-unknown-linux-musl

# 第二阶段：运行（纯净Alpine）
FROM alpine:latest
WORKDIR /app
EXPOSE 6100
LABEL maintainer="lkhsss1019@gmail.com"
LABEL version="1.0"
LABEL description="qq-banner"
# 从构建阶段复制静态链接的可执行文件
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/qq-banner /app/

CMD ["./qq-banner"]