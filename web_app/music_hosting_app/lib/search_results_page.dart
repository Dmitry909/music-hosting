import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;
import 'dart:convert';

import 'track_widget.dart';

class SearchResultsPage extends StatefulWidget {
  final String query;

  SearchResultsPage({Key? key, required this.query}) : super(key: key);

  @override
  _SearchResultsPageState createState() => _SearchResultsPageState();
}

class _SearchResultsPageState extends State<SearchResultsPage> {
  late Future<List<String>> _idsFuture;

  @override
  void initState() {
    super.initState();
    _idsFuture = _fetchIds(widget.query);
  }

  Future<List<String>> _fetchIds(String query) async {
    final response =
        await http.get(Uri.parse('http://localhost:3000/search?query=$query'));

    if (response.statusCode == 200) {
      List<dynamic> data = jsonDecode(response.body);
      return List<String>.from(data.map((id) => id.toString()));
    } else {
      throw Exception('Failed to load Ids');
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Search Results for "${widget.query}"'),
      ),
      body: FutureBuilder<List<String>>(
        future: _idsFuture,
        builder: (context, snapshot) {
          if (snapshot.connectionState == ConnectionState.waiting) {
            return Center(child: CircularProgressIndicator());
          } else if (snapshot.hasError) {
            return Center(child: Text("Error: ${snapshot.error}"));
          }

          final ids = snapshot.data ?? [];
          return ListView.builder(
            itemCount: ids.length,
            itemBuilder: (context, index) {
              // return ListTile(
              //   title: Text(ids[index]),
              // );
              print(ids[index]);
              return TrackWidget(
                id: 123,
                authorUsername: "Vasya",
                name: "Name of the track",
                cntRates: 0,
                sumRates: 0,
              );
            },
          );
        },
      ),
    );
  }
}
