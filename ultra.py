#!/usr/bin/python3
import RPi.GPIO as GPIO
import time, gc, os

GPIO.setmode(GPIO.BOARD)
PIN_TRIGGER = 18  # 18? # 24
PIN_ECHO = 11  # 11  #17
GPIO.setup(PIN_TRIGGER, GPIO.OUT)
GPIO.setup(PIN_ECHO, GPIO.IN)

def distance() -> int:

    GPIO.output(PIN_TRIGGER, GPIO.LOW)

    GPIO.output(PIN_TRIGGER, GPIO.HIGH)

    time.sleep(0.00001)

    GPIO.output(PIN_TRIGGER, GPIO.LOW)

    while GPIO.input(PIN_ECHO) == 0:
        pulse_start_time = time.time()
    while GPIO.input(PIN_ECHO) == 1:
        pulse_end_time = time.time()

    pulse_duration = pulse_end_time - pulse_start_time
    distance = round(pulse_duration * 17150, 2)
    return distance


if __name__ == '__main__':
    try:
        while True:
            dist = distance()
            with open("./distance", 'w') as f:
                f.write(str(dist))
                f.flush
                os.fsync(f.fileno())
            gc.collect()
            time.sleep(5)

        # Reset by pressing CTRL + C
    except KeyboardInterrupt:
        GPIO.cleanup()
        print("Measurement stopped by User")
    except Exception as e:
        GPIO.cleanup()
        print(e)
