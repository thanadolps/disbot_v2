FROM python:3-alphine

WORKDIR /app
COPY ./target/release/disbot_v2 ./disbot_v2
COPY ./python_dir ./python_dir

# Define environment variable
# This is a placeholder - the actual token should be passed at runtime
ENV DISCORD_TOKEN="placeholder"

CMD ["./disbot_v2"]