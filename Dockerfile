FROM debian:bookworm-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    sudo \
    jq \
    ca-certificates && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Fetch latest Tooka release and install
RUN LATEST_URL=$(curl -s https://api.github.com/repos/Benji377/tooka/releases/latest | \
    jq -r '.assets[] | select(.name | endswith("_amd64.deb")) | .browser_download_url') && \
    wget -O tooka.deb "$LATEST_URL" && \
    sudo dpkg -i tooka.deb && \
    rm tooka.deb

# Create non-root user
RUN useradd -ms /bin/bash tooka
USER tooka

# Create workspace directory
WORKDIR /home/tooka/workspace

# Entrypoint to keep container alive with bash shell
ENTRYPOINT ["/bin/bash"]
