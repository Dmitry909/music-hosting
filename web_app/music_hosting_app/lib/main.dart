import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import 'main_page.dart';
import 'shared_state.dart';

void main() {
  runApp(ChangeNotifierProvider(
    create: (context) => QueueModel(),
    child: MusicHostingApp(),
  ));
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
