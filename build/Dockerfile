FROM rust:1.89

# Install system dependencies for Bevy
RUN apt-get update && apt-get install -y \
    pkg-config \
    libx11-dev \
    libasound2-dev \
    libudev-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev \
    libwayland-dev \
    libxkbcommon-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /workspace

# Copy CI commands script
COPY ci-commands.sh /usr/local/bin/ci-commands.sh
RUN chmod +x /usr/local/bin/ci-commands.sh

# Default command runs full CI
CMD ["/usr/local/bin/ci-commands.sh"]
