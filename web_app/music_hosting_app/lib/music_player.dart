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
  int? currentTrackId;

  @override
  void initState() {
    super.initState();
    audioPlayer.onDurationChanged.listen((Duration d) {
      setState(() {
        duration = d;
      });
    });
    // audioPlayer.onAudioPositionChanged.listen((Duration p) {
    //   setState(() {
    //     position = p;
    //   });
    // });
  }

  @override
  void dispose() {
    audioPlayer.dispose();
    super.dispose();
  }

  void playMusic(queueModel) async {
    print('Called playMusic');
    currentTrackId ??= queueModel.removeFromQueue();
    int trackId = currentTrackId ?? 0;
    await audioPlayer
        .play(UrlSource('http://localhost:3000/download_track?id=$trackId'));
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
    // Implement previous track logic
  }

  void nextTrack() {
    // Implement next track logic
  }

  String formatTime(Duration duration) {
    String twoDigits(int n) => n.toString().padLeft(2, "0");
    String twoDigitMinutes = twoDigits(duration.inMinutes.remainder(60));
    String twoDigitSeconds = twoDigits(duration.inSeconds.remainder(60));
    return "$twoDigitMinutes:$twoDigitSeconds";
  }

  @override
  Widget build(BuildContext context) {
    final queueModel = Provider.of<QueueModel>(context);

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
                    playMusic(queueModel);
                  },
          ),
          IconButton(
            icon: const Icon(Icons.skip_next),
            onPressed: nextTrack,
          ),
          Expanded(
            child: Slider(
              value: position.inSeconds.toDouble(),
              min: 0.0,
              max: duration.inSeconds.toDouble(),
              onChanged: (double value) {
                audioPlayer.seek(Duration(seconds: value.toInt()));
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
