# A partir de la imagen oficial de Rust
FROM rust as constructor

# Copio el código del proyecto y lo pego en la imagen
COPY . /app

# Establezco el directorio de trabajo al del proyecto
WORKDIR /app

# Construyo el programa
RUN cargo build --release

# Ejecuto el programa
CMD ["./target/release/servidor"]