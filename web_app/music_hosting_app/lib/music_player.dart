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

  void playMusic(playerData) async {
    print('Called playMusic');
    final currentTrackId = playerData.getCurrentTrackId();
    if (currentTrackId == -1) {
      return;
    }
    await audioPlayer.play(
        UrlSource('http://localhost:3000/download_track?id=$currentTrackId'));
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

  void previousTrack() {
    // Would be implemented
  }

  void nextTrack(playerData) {
    playerData.goToNextTrack();
  }

  String formatTime(Duration duration) {
    String twoDigits(int n) => n.toString().padLeft(2, "0");
    String twoDigitMinutes = twoDigits(duration.inMinutes.remainder(60));
    String twoDigitSeconds = twoDigits(duration.inSeconds.remainder(60));
    return "$twoDigitMinutes:$twoDigitSeconds";
  }

  @override
  Widget build(BuildContext context) {
    final playerData = Provider.of<PlayerData>(context);

    if (playerData.releaseNewTrackAdded()) {
      playMusic(playerData);
    }

    return BottomAppBar(
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: [
          IconButton(
            icon: const Icon(Icons.skip_previous),
            onPressed: previousTrack,
          ),
          IconButton(
            icon: Icon(isPlaying ? Icons.pause : Icons.play_arrow),
            onPressed: isPlaying
                ? pauseMusic
                : () {
                    playMusic(playerData);
                  },
          ),
          IconButton(
            icon: const Icon(Icons.skip_next),
            onPressed: () {
              nextTrack(playerData);
            },
          ),
          Expanded(
            child: Slider(
              value: position.inSeconds.toDouble(),
              min: 0.0,
              max: duration.inSeconds.toDouble(),
              onChanged: (double value) {
                // audioPlayer.seek(Duration(seconds: value.toInt()));
              },
            ),
          ),
          Text(
            '${formatTime(position)} / ${formatTime(duration)}',
            style: const TextStyle(fontSize: 16),
          ),
        ],
      ),
    );
  }
}
