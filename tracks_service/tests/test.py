from common import *


def upload_track_and_delete_account():
    id_a = upload_track('alex', 'Porokh', 'test_tracks/a.mp3')
    id_b = upload_track('alex', 'Mimino', 'test_tracks/b.mp3')
    id_c = upload_track('alex', 'Pelevino', 'test_tracks/c.mp3')

    delete_account('alex', [id_a, id_b, id_c])
    print('upload_track_and_delete_account success')


def upload_delete_track_and_account():
    id_a = upload_track('alex', 'Porokh', 'test_tracks/a.mp3')
    id_b = upload_track('alex', 'Mimino', 'test_tracks/b.mp3')
    id_c = upload_track('alex', 'Pelevino', 'test_tracks/c.mp3')

    delete_track('alex', id_a)
    assert (os.path.exists(f'../tracks/{id_b}.mp3'))
    assert (os.path.exists(f'../tracks/{id_c}.mp3'))

    delete_account('alex', [id_b, id_c])
    print('upload_delete_track_and_account success')


upload_track_and_delete_account()
upload_delete_track_and_account()
