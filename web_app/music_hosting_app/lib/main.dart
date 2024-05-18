import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'main_page.dart';
import 'shared_state.dart';

void main() {
  runApp(ChangeNotifierProvider(
    create: (context) => PlayerData(),
    child: const MusicHostingApp(),
  ));
}

class MusicHostingApp extends StatelessWidget {
  const MusicHostingApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Application',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: const MainPage(),
    );
  }
}
