FROM python:3-slim

WORKDIR /app
COPY ./target/release/disbot_v2 ./disbot_v2
COPY ./python_dir ./python_dir


# fix ./disbot_v2: /lib/aarch64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found (required by ./disbot_v2)
RUN apt-get update && apt-get install -y build-essential && apt-get upgrade -y

# Define environment variable
# This is a placeholder - the actual token should be passed at runtime
ENV DISCORD_TOKEN="placeholder"

RUN chmod +x disbot_v2

CMD ["./disbot_v2"]