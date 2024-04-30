import 'package:flutter/material.dart';

import 'main_page.dart';

void main() {
  runApp(MusicHostingApp());
}

class MusicHostingApp extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Application',
      theme: ThemeData(
        primarySwatch: Colors.blue,
      ),
      home: MainPage(),
    );
  }
}
