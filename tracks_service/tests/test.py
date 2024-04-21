from common import *
import random
from string import ascii_uppercase, digits


def random_str(length):
    return ''.join(random.choice(ascii_uppercase + digits) for _ in range(length))


def test_upload_track_and_delete_account():
    account = random_str(10)
    id_a = upload_track(account, random_str(10), 'test_tracks/a.mp3')
    id_b = upload_track(account, random_str(10), 'test_tracks/b.mp3')
    id_c = upload_track(account, random_str(10), 'test_tracks/c.mp3')

    delete_account(account, [id_a, id_b, id_c])
    print('test_upload_track_and_delete_account success')


def test_upload_delete_track_and_account():
    account = random_str(10)
    id_a = upload_track(account, random_str(10), 'test_tracks/a.mp3')
    id_b = upload_track(account, random_str(10), 'test_tracks/b.mp3')
    id_c = upload_track(account, random_str(10), 'test_tracks/c.mp3')

    delete_track(account, id_a)
    assert (os.path.exists(f'../tracks/{id_b}.mp3'))
    assert (os.path.exists(f'../tracks/{id_c}.mp3'))

    delete_account(account, [id_b, id_c])
    print('test_upload_delete_track_and_account success')


def test_upload_and_download():
    account = random_str(10)
    file_path = 'test_tracks/a.mp3'

    id = upload_track(account, random_str(10), file_path)
    track_content_got = download_track(id)
    with open(file_path, 'rb') as f:
        track_content_real = f.read()
        assert (track_content_got == track_content_real)

    print('test_upload_and_download success')


def test_search():
    account1 = random_str(10)
    account2 = random_str(10)
    common_part = random_str(10)

    id_a = upload_track(account1, f'{common_part}cde', 'test_tracks/a.mp3')
    id_b = upload_track(account2, f'q{common_part}we', 'test_tracks/b.mp3')
    id_c = upload_track(account2, f'q{common_part}cq', 'test_tracks/c.mp3')

    search(common_part, {id_a, id_b, id_c})
    search(f'q{common_part}', {id_b, id_c})
    search(f'q{common_part}cq', {id_c})
    search(f'{common_part}c', {id_a, id_c})

    delete_account(account1, [id_a])
    delete_account(account2, [id_b, id_c])

    print('test_search success')


test_upload_track_and_delete_account()
test_upload_delete_track_and_account()
test_upload_and_download()
test_search()
