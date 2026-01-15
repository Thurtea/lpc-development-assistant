FROM mcr.microsoft.com/vscode/devcontainers/rust:1

# Install Node.js (LTS) for the UI
RUN su vscode -c "wget https://nodejs.org/dist/v18.18.0/node-v18.18.0-linux-x64.tar.xz -O /tmp/node.tar.xz && mkdir -p /home/vscode/node && tar -xf /tmp/node.tar.xz -C /home/vscode/node --strip-components=1 && rm /tmp/node.tar.xz"
ENV PATH=/home/vscode/node/bin:$PATH
