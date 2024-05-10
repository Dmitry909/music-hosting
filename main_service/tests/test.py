from common import *
import random
from string import ascii_uppercase, digits


def random_str(length):
    return ''.join(random.choice(ascii_uppercase + digits) for _ in range(length))


def test_login_logout():
    username = random_str(10)
    password = random_str(10)

    signup_resp = signup(username, password)
    assert (signup_resp.status_code == 201)

    login_resp = login(username, password)
    token = login_resp.headers["Authorization"]
    assert (len(token) > 10 and len(token) < 1000)

    logout_resp = logout(token)
    assert (logout_resp.status_code == 200)

    delete_account(username, password)

    print('test_signup_login_logout OK')


def test_login_check_token():
    username = random_str(10)
    password = random_str(10)
    signup(username, password)
    token = login(username, password).headers["Authorization"]

    check_token_resp1 = check_token(token)
    assert (check_token_resp1.status_code == 200)

    check_token_resp2 = check_token('fake_token')
    assert (check_token_resp2.status_code == 401)

    delete_account(username, password)

    print('test_login_check_token OK')


def test_upload_download():
    username = random_str(10)
    password = random_str(10)
    track_name = random_str(10)
    file_path = "test_tracks/fake.mp3"

    signup(username, password)
    login_resp = login(username, password)
    token = login_resp.headers["Authorization"]

    track_id = upload_track(token, track_name, file_path)
    download_resp = download_track(track_id)
    assert (download_resp.status_code == 200)
    track_content_got = download_resp.content
    with open(file_path, 'rb') as f:
        track_content_real = f.read()
        assert (track_content_got == track_content_real)

    delete_account(username, password)

    print('test_upload_download OK')


def test_upload_delete():
    username = random_str(10)
    password = random_str(10)
    track_name = random_str(10)
    file_path = "test_tracks/a.mp3"

    signup(username, password)
    login_resp = login(username, password)
    token = login_resp.headers["Authorization"]

    track_id = upload_track(token, track_name, file_path)

    delete_track_resp = delete_track(token, track_id)
    assert (delete_track_resp.status_code == 200)

    download_resp = download_track(track_id)
    assert (download_resp.status_code == 404)

    delete_account(username, password)

    print('test_upload_delete OK')


def test_upload_delete_account():
    username = random_str(10)
    password = random_str(10)
    track_name = random_str(10)
    file_path = "test_tracks/a.mp3"

    signup(username, password)
    login_resp = login(username, password)
    token = login_resp.headers["Authorization"]

    track_id = upload_track(token, track_name, file_path)

    delete_account_resp = delete_account(username, password)
    assert (delete_account_resp.status_code == 200)

    download_resp = download_track(track_id)
    assert (download_resp.status_code == 404)

    login_resp = login(username, password)
    assert (login_resp.status_code == 404)

    print('test_upload_delete_account OK')


def test_create_delete_get_playlist():
    username = random_str(10)
    password = random_str(10)
    playlist_name = random_str(10)

    signup(username, password)
    token = login(username, password).headers["Authorization"]

    playlist_id = create_playlist(token, playlist_name)

    add_to_playlist(token, playlist_id, 123)

    tracks = get_playlist(playlist_id)
    assert (tracks == [123])

    delete_playlist_resp = delete_playlist(token, playlist_id)
    assert (delete_playlist_resp.status_code == 200)

    # track_name = random_str(10)
    # file_path = "test_tracks/a.mp3"

    print('test_create_delete_get_playlist OK')


test_login_logout()
test_login_check_token()
test_upload_download()
test_upload_delete()
test_upload_delete_account()
test_create_delete_get_playlist()
