# Start from Arch Linux base image
FROM archlinux:latest

# Install dependencies
RUN pacman -Syu --noconfirm archiso git base-devel grub

# Set workdir
WORKDIR /iso

# Create necessary directories
RUN mkdir -p p out w

# Copy baseline ArchISO configuration
RUN cp -r /usr/share/archiso/configs/baseline/* p/

# Create root folder inside ISO
RUN mkdir -p p/airootfs/root

# Copy Flutter app bundle into ISO root (from build context)
COPY bundle/* p/airootfs/root/

# Copy packages.x86_64 and profiledef.sh from host into ISO
COPY packages.x86_64 p/
COPY profiledef.txt p/profiledef.sh

CMD ["bash"]
