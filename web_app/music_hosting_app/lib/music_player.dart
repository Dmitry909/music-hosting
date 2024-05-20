import 'package:flutter/material.dart';
import 'package:audioplayers/audioplayers.dart';
import 'package:provider/provider.dart';

import 'shared_state.dart';

class MusicPlayer extends StatefulWidget {
  const MusicPlayer({super.key});

  @override
  _MusicPlayerState createState() => _MusicPlayerState();
}

class _MusicPlayerState extends State<MusicPlayer> {
  AudioPlayer audioPlayer = AudioPlayer();
  bool isPlaying = false;
  Duration duration = const Duration();
  Duration position = const Duration();
  String trackName = "";
  String authorUsername = "";

  @override
  void initState() {
    super.initState();
    audioPlayer.onDurationChanged.listen((Duration d) {
      print('Called audioPlayer.onDurationChanged');
      setState(() {
        duration = d;
      });
    });
    audioPlayer.onPositionChanged.listen((Duration p) {
      setState(() {
        position = p;
      });
    });
    audioPlayer.onPlayerComplete.listen((event) {
      print('Called audioPlayer.onPlayerComplete');
      setState(() {
        isPlaying = false;
        position = const Duration();
      });
    });
  }

  @override
  void dispose() {
    audioPlayer.dispose();
    super.dispose();
  }

  void playMusic(currentTrackInfo) async {
    print('Called playMusic');
    if (currentTrackInfo.id == -1) {
      return;
    }

    await audioPlayer.play(UrlSource(
        'http://localhost:3000/download_track?id=${currentTrackInfo.id}'));
    setState(() {
      isPlaying = true;
    });
  }

  void pauseMusic() async {
    await audioPlayer.pause();
    setState(() {
      isPlaying = false;
    });
  }

  String formatTime(Duration duration) {
    String twoDigits(int n) => n.toString().padLeft(2, "0");
    String twoDigitMinutes = twoDigits(duration.inMinutes.remainder(60));
    String twoDigitSeconds = twoDigits(duration.inSeconds.remainder(60));
    return "$twoDigitMinutes:$twoDigitSeconds";
  }

  Widget buildMusicPlayerRow(playerData, pos, dur) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
      children: [
        Container(
          width: 200.0,
          child: Column(
            children: [
              Text(
                trackName,
                style: TextStyle(fontSize: 10),
              ),
              Text(
                authorUsername,
                style: TextStyle(fontSize: 10),
              ),
            ],
          ),
        ),
        IconButton(
          icon: const Icon(Icons.skip_previous),
          onPressed: () {
            playerData.goToPreviousTrack();
          },
        ),
        IconButton(
          icon: Icon(isPlaying ? Icons.pause : Icons.play_arrow),
          onPressed: isPlaying
              ? pauseMusic
              : () {
                  playMusic(playerData.getCurrentTrackInfo());
                },
        ),
        IconButton(
          icon: const Icon(Icons.skip_next),
          onPressed: () {
            playerData.goToNextTrack();
          },
        ),
        Expanded(
          child: Slider(
            value: pos.inSeconds.toDouble(),
            min: 0.0,
            max: dur.inSeconds.toDouble(),
            onChanged: (double value) {
              // audioPlayer.seek(Duration(seconds: value.toInt()));
            },
          ),
        ),
        Text(
          '${formatTime(pos)} / ${formatTime(dur)}',
          style: const TextStyle(fontSize: 16),
        ),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    final playerData = Provider.of<PlayerData>(context);

    if (playerData.releaseNewTrackAdded()) {
      final currentTrackInfo = playerData.getCurrentTrackInfo();
      setState(() {
        trackName = currentTrackInfo.name;
        if (trackName.length > 20) {
          trackName = trackName.substring(0, 20) + "...";
        }
        authorUsername = currentTrackInfo.authorUsername;
        if (authorUsername.length > 20) {
          authorUsername = authorUsername.substring(0, 20) + "...";
        }
      });
      playMusic(currentTrackInfo);
    }

    var pos = position;
    final dur = duration;
    if (pos.inSeconds.toDouble() > 0.0 && dur.inSeconds.toDouble() == 0.0) {
      pos = Duration.zero;
    }

    return BottomAppBar(
      child: Column(
        children: [
          buildMusicPlayerRow(playerData, pos, dur),
        ],
      ),
    );
  }
}
