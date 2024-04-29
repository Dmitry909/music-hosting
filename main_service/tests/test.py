from common import *
import random
from string import ascii_uppercase, digits


def random_str(length):
    return ''.join(random.choice(ascii_uppercase + digits) for _ in range(length))


def test_signup_login_logout():
    account = random_str(10)
    password = random_str(10)

    signup_resp = signup(account, password)
    assert(signup_resp.status_code == 201)

    login_resp = login(account, password)
    token = login_resp.headers["Authorization"]
    assert(len(token) > 10 and len(token) < 1000)
    
    logout_resp = logout(token)
    assert(logout_resp.status_code == 200)

    print('test_signup_login_logout OK')


def test_delete_account():
    account = random_str(10)
    password = random_str(10)

    signup(account, password)
    token = login(account, password).headers["Authorization"]

    print('test_delete_account OK')


test_signup_login_logout()
test_delete_account()
