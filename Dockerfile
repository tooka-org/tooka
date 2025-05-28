FROM debian:bookworm-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    sudo \
    ca-certificates && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Download and install Tooka
RUN wget -O tooka.deb https://github.com/Benji377/tooka/releases/download/v0.1.1/tooka_0.1.1_amd64.deb && \
    sudo dpkg -i tooka.deb && \
    rm tooka.deb

# Create non-root user
RUN useradd -ms /bin/bash tooka
USER tooka

# Create workspace directory
WORKDIR /home/tooka/workspace

# Entrypoint to keep container alive with bash shell
ENTRYPOINT ["/bin/bash"]
