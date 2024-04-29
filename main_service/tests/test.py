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


def test_upload_download():
    account = random_str(10)
    password = random_str(10)
    track_name = random_str(10)
    file_path = "test_tracks/fake.mp3"

    signup(account, password)
    login_resp = login(account, password)
    token = login_resp.headers["Authorization"]
    
    track_id = upload_track(token, account, track_name, file_path)
    download_resp = download_track(track_id)
    assert(download_resp.status_code == 200)
    track_content_got = download_resp.content
    with open(file_path, 'rb') as f:
        track_content_real = f.read()
        assert(track_content_got == track_content_real)
    
    print('test_upload_download OK')


def test_upload_delete():
    account = random_str(10)
    password = random_str(10)
    track_name = random_str(10)
    file_path = "test_tracks/a.mp3"

    signup(account, password)
    login_resp = login(account, password)
    token = login_resp.headers["Authorization"]
    
    track_id = upload_track(token, account, track_name, file_path)

    delete_track_resp = delete_track(token, account, track_id)
    assert(delete_track_resp.status_code == 200)

    download_resp = download_track(track_id)
    assert(download_resp.status_code == 404)

    print('test_upload_delete OK')


def test_upload_delete_account():
    account = random_str(10)
    password = random_str(10)
    track_name = random_str(10)
    file_path = "test_tracks/a.mp3"

    signup(account, password)
    login_resp = login(account, password)
    token = login_resp.headers["Authorization"]
    
    track_id = upload_track(token, account, track_name, file_path)

    delete_account_resp = delete_account(account, password)
    assert(delete_account_resp.status_code == 200)

    download_resp = download_track(track_id)
    assert(download_resp.status_code == 404)

    login_resp = login(account, password)
    assert(login_resp.status_code == 404)

    print('test_upload_delete_account OK')


test_signup_login_logout()
test_upload_download()
test_upload_delete()
test_upload_delete_account()
