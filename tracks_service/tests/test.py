from common import *


def test_upload_track_and_delete_account():
    id_a = upload_track('alex', 'Porokh', 'test_tracks/a.mp3')
    id_b = upload_track('alex', 'Mimino', 'test_tracks/b.mp3')
    id_c = upload_track('alex', 'Pelevino', 'test_tracks/c.mp3')

    delete_account('alex', [id_a, id_b, id_c])
    print('test_upload_track_and_delete_account success')


def test_upload_delete_track_and_account():
    id_a = upload_track('alex', 'Porokh', 'test_tracks/a.mp3')
    id_b = upload_track('alex', 'Mimino', 'test_tracks/b.mp3')
    id_c = upload_track('alex', 'Pelevino', 'test_tracks/c.mp3')

    delete_track('alex', id_a)
    assert (os.path.exists(f'../tracks/{id_b}.mp3'))
    assert (os.path.exists(f'../tracks/{id_c}.mp3'))

    delete_account('alex', [id_b, id_c])
    print('test_upload_delete_track_and_account success')


def test_upload_and_download():
    delete_account('alex', [])
    file_path = 'test_tracks/a.mp3'
    id = upload_track('alex', 'Porokh', file_path)
    track_content_got = download_track(id)
    with open(file_path, 'rb') as f:
        track_content_real = f.read()
        assert (track_content_got == track_content_real)

    print('test_upload_and_download success')


test_upload_track_and_delete_account()
test_upload_delete_track_and_account()
test_upload_and_download()
