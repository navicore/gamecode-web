#!/usr/bin/env python3
import argon2
import sys

password = sys.argv[1] if len(sys.argv) > 1 else "gamecode"
ph = argon2.PasswordHasher()
hash = ph.hash(password)
print(f"Password: {password}")
print(f"Hash: {hash}")