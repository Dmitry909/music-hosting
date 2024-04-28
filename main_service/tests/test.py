from common import *
import random
from string import ascii_uppercase, digits


def random_str(length):
    return ''.join(random.choice(ascii_uppercase + digits) for _ in range(length))


def test_signup_login_logout():
    account = random_str(10)
    password = random_str(10)
    signup(account, password)
    token = login(account, password) 
    assert(len(token) > 10 and len(token) < 1000)
    logout(token)
    print('test_signup_login_logout success')


test_signup_login_logout()
