"""
Video to ASCII converter for RustOS
"""

import cv2
import argparse
import os

ASCII_CHARS = " .:+#@"

def main():
    parser = argparse.ArgumentParser(description="Convert video to ASCII for RustOS")
    parser.add_argument("video", help="Path to input video file")
    parser.add_argument("--width", type=int, default=80, help="ASCII width")
    parser.add_argument("--height", type=int, default=24, help="ASCII height")
    parser.add_argument("--fps", type=int, default=15, help="Target FPS")
    parser.add_argument("--output", default="src/bad_apple_data.rs", help="Output file")
    parser.add_argument("--invert", action="store_true", help="Invert colors (white on black)")
    args = parser.parse_args()
    
    WIDTH = args.width
    HEIGHT = args.height
    TARGET_FPS = args.fps
    FRAME_SIZE = WIDTH * HEIGHT
    
    cap = cv2.VideoCapture(args.video)
    if not cap.isOpened():
        print("Error: Cannot open video")
        return
    
    original_fps = cap.get(cv2.CAP_PROP_FPS)
    frame_skip = max(1, int(original_fps / TARGET_FPS))
    
    print(f"Video FPS: {original_fps}, Target: {TARGET_FPS}, Skip: {frame_skip}")
    print(f"Output size: {WIDTH}x{HEIGHT} = {FRAME_SIZE} bytes per frame")
    
    # Collect all frames as flat byte strings
    all_frames = []
    frame_num = 0
    
    while True:
        ret, frame = cap.read()
        if not ret:
            break
        
        if frame_num % frame_skip == 0:
            # Convert to grayscale and resize
            gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
            resized = cv2.resize(gray, (WIDTH, HEIGHT))
            
            # Invert if requested (Bad Apple has white figure on black)
            if args.invert:
                resized = 255 - resized
            
            # Convert to ASCII - build exactly FRAME_SIZE characters
            chars = []
            for y in range(HEIGHT):
                for x in range(WIDTH):
                    pixel = resized[y, x]
                    idx = min(int(pixel / 256 * len(ASCII_CHARS)), len(ASCII_CHARS) - 1)
                    chars.append(ASCII_CHARS[idx])
            
            # Verify exact size
            frame_str = ''.join(chars)
            assert len(frame_str) == FRAME_SIZE, f"Frame size mismatch: {len(frame_str)} != {FRAME_SIZE}"
            all_frames.append(frame_str)
            
            if len(all_frames) % 100 == 0:
                print(f"Processed {len(all_frames)} frames...")
        
        frame_num += 1
    
    cap.release()
    print(f"Total frames: {len(all_frames)}")
    
    # Write Rust file
    with open(args.output, 'w') as f:
        f.write("// Auto-generated video ASCII frames\n")
        f.write("// Do not edit manually!\n\n")
        f.write(f"pub const FRAME_WIDTH: usize = {WIDTH};\n")
        f.write(f"pub const FRAME_HEIGHT: usize = {HEIGHT};\n")
        f.write(f"pub const FRAME_COUNT: usize = {len(all_frames)};\n")
        f.write(f"pub const TARGET_FPS: u32 = {TARGET_FPS};\n\n")
        
        # Write all frames as a byte array (avoids escaping issues)
        f.write("pub static FRAMES: &[u8] = &[\n")
        
        for i, frame_data in enumerate(all_frames):
            # Convert each character to its byte value
            f.write("    ")
            for j, char in enumerate(frame_data):
                f.write(f"{ord(char)}")
                if j < len(frame_data) - 1:
                    f.write(",")
                # Add line breaks every 80 characters for readability
                if (j + 1) % 80 == 0:
                    f.write("\n    ")
            
            # Add comma between frames
            if i < len(all_frames) - 1:
                f.write(",\n")
            else:
                f.write("\n")
        
        f.write("];\n")
    
    total_bytes = len(all_frames) * FRAME_SIZE
    file_size = os.path.getsize(args.output)
    print(f"Generated: {args.output}")
    print(f"Expected data: {total_bytes} bytes ({len(all_frames)} frames x {FRAME_SIZE})")
    print(f"File size: {file_size / 1024:.1f} KB")

if __name__ == "__main__":
    main()
