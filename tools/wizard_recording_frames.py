"""One explicit 60 Hz sampling stream for measuring and exporting VFR recordings."""
import subprocess

FPS, WIDTH, HEIGHT = 60, 1280, 900


def frames(ffmpeg, path):
    process = subprocess.Popen([
        ffmpeg, '-hide_banner', '-loglevel', 'error', '-i', str(path),
        '-vf', f'fps={FPS}', '-fps_mode', 'passthrough',
        '-f', 'rawvideo', '-pix_fmt', 'rgb24', '-'], stdout=subprocess.PIPE)
    try:
        while raw := process.stdout.read(WIDTH * HEIGHT * 3):
            if len(raw) != WIDTH * HEIGHT * 3:
                raise AssertionError('partial decoded sample')
            yield raw
        if process.wait():
            raise RuntimeError('video decoder failed')
    finally:
        if process.poll() is None:
            process.terminate()
        process.wait()
        process.stdout.close()
